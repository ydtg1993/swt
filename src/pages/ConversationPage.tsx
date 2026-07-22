import { useState, useRef, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "@/hooks/useI18n";
import { invokeCmd } from "@/hooks/useInvoke";

interface Turn {
  role: "user" | "assistant";
  text: string;
  audioPath?: string;
}

interface ConvEvent {
  event_type: string;
  text: string;
  audio_path: string | null;
}

function encodeWav(samples: Int16Array, sampleRate: number): ArrayBuffer {
  const byteLength = samples.length * 2;
  const buffer = new ArrayBuffer(44 + byteLength);
  const view = new DataView(buffer);
  const ws = (o: number, s: string) => { for (let i = 0; i < s.length; i++) view.setUint8(o + i, s.charCodeAt(i)); };
  ws(0, "RIFF"); view.setUint32(4, 36 + byteLength, true); ws(8, "WAVE");
  ws(12, "fmt "); view.setUint32(16, 16, true); view.setUint16(20, 1, true); view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true); view.setUint32(28, sampleRate * 2, true); view.setUint16(32, 2, true); view.setUint16(34, 16, true);
  ws(36, "data"); view.setUint32(40, byteLength, true);
  (new Uint8Array(buffer, 44)).set(new Uint8Array(samples.buffer, samples.byteOffset, samples.byteLength));
  return buffer;
}

export default function ConversationPage() {
  const { t } = useI18n();
  const [turns, setTurns] = useState<Turn[]>([]);
  const [listening, setListening] = useState(false);
  const [status, setStatus] = useState("");
  const [currentText, setCurrentText] = useState("");
  const speechBufferRef = useRef<Int16Array[]>([]);
  const isSpeakingRef = useRef(false);
  const silenceFramesRef = useRef(0);
  const streamRef = useRef<{ ctx: AudioContext; proc: ScriptProcessorNode; stream: MediaStream } | null>(null);

  useEffect(() => {
    const unlisten = listen<ConvEvent>("conversation-event", (e) => {
      const ev = e.payload;
      if (ev.event_type === "user_text") {
        setTurns((prev) => [...prev, { role: "user", text: ev.text }]);
        setCurrentText("");
      } else if (ev.event_type === "assistant_text") {
        setTurns((prev) => [...prev, { role: "assistant", text: ev.text, audioPath: ev.audio_path ?? undefined }]);
        setStatus("");
      } else if (ev.event_type === "status") {
        setStatus(ev.text);
      } else if (ev.event_type === "error") {
        setStatus(ev.text);
      }
    });
    return () => {
      unlisten.then((fn) => fn());
      // Clean up mic resources on unmount (navigating away while listening)
      if (streamRef.current) {
        streamRef.current.proc.disconnect();
        streamRef.current.stream.getTracks().forEach((t) => t.stop());
        void streamRef.current.ctx.close();
        streamRef.current = null;
      }
    };
  }, []);

  const flushAndSend = async () => {
    if (speechBufferRef.current.length < 10) {
      speechBufferRef.current = []; isSpeakingRef.current = false; return;
    }
    const totalLen = speechBufferRef.current.reduce((a, b) => a + b.length, 0);
    const combined = new Int16Array(totalLen);
    let offset = 0;
    for (const b of speechBufferRef.current) { combined.set(b, offset); offset += b.length; }
    const wav = encodeWav(combined, 16000);
    speechBufferRef.current = []; silenceFramesRef.current = 0; isSpeakingRef.current = false;
    setStatus("识别中...");
    try {
      await invokeCmd("conversation_turn", {
        wavBytes: Array.from(new Uint8Array(wav)),
        language: "zh",
        conversationHistory: [],
      });
    } catch (e) { setStatus(`失败: ${e}`); }
  };

  const processAudio = (samples: Int16Array) => {
    let sum = 0; for (let i = 0; i < samples.length; i++) sum += samples[i] * samples[i];
    const energy = Math.sqrt(sum / samples.length);
    if (energy > 50) {
      isSpeakingRef.current = true; silenceFramesRef.current = 0;
      speechBufferRef.current.push(samples);
    } else if (isSpeakingRef.current) {
      silenceFramesRef.current++;
      speechBufferRef.current.push(samples);
      if (silenceFramesRef.current > 25 && speechBufferRef.current.length > 15) {
        void flushAndSend();
      }
    }
  };

  const toggleMic = async () => {
    if (listening) {
      if (isSpeakingRef.current) void flushAndSend();
      if (streamRef.current) {
        streamRef.current.proc.disconnect();
        streamRef.current.stream.getTracks().forEach((t) => t.stop());
        void streamRef.current.ctx.close();
      }
      setListening(false);
    } else {
      try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: { sampleRate: 16000, channelCount: 1 } });
        const ctx = new AudioContext({ sampleRate: 16000 });
        const source = ctx.createMediaStreamSource(stream);
        const proc = ctx.createScriptProcessor(1024, 1, 1);
        source.connect(proc); proc.connect(ctx.destination);
        proc.onaudioprocess = (e) => {
          const input = e.inputBuffer.getChannelData(0);
          const int16 = new Int16Array(input.length);
          for (let i = 0; i < input.length; i++) int16[i] = Math.max(-32768, Math.min(32767, Math.round(input[i] * 32767)));
          processAudio(int16);
        };
        streamRef.current = { ctx, proc, stream };
        setListening(true);
      } catch (e) { setStatus(`麦克风失败: ${e}`); }
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-bold">{t("nav.conversation")}</h2>
      <div className="flex items-center gap-3">
        <button onClick={toggleMic}
          className={`rounded-full px-6 py-3 font-bold ${listening ? "bg-destructive text-destructive-foreground animate-pulse" : "bg-primary text-primary-foreground"}`}>
          {listening ? "⏹ 停止" : "🎙️ 开始对话"}
        </button>
        {status && <span className="text-sm text-muted-foreground">{status}</span>}
      </div>
      <div className="space-y-3 max-h-96 overflow-y-auto">
        {turns.map((turn, i) => (
          <div key={i} className={`flex ${turn.role === "user" ? "justify-end" : "justify-start"}`}>
            <div className={`max-w-[80%] rounded-xl px-4 py-3 ${turn.role === "user" ? "bg-primary text-primary-foreground" : "bg-muted"}`}>
              <p className="text-sm">{turn.text}</p>
              {turn.audioPath && (
                <audio controls className="mt-2 h-8 w-full" src={`asset://localhost/${turn.audioPath}`} />
              )}
            </div>
          </div>
        ))}
        {turns.length === 0 && (
          <div className="rounded-xl border border-border p-12 text-center text-muted-foreground">
            <p className="text-4xl mb-2">💬</p>
            <p>点击麦克风按钮开始语音对话</p>
            <p className="text-xs mt-2">VAD → ASR → LLM → TTS</p>
          </div>
        )}
        {currentText && <div className="text-center text-sm text-muted-foreground italic">{currentText}</div>}
      </div>
    </div>
  );
}
