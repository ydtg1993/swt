import { useState, useRef, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "@/hooks/useI18n";
import { invokeCmd } from "@/hooks/useInvoke";

interface Segment {
  text: string;
  start: number;
  end: number;
  confidence: number;
}

interface StreamEvent {
  event_type: string;
  text: string;
  start: number | null;
  end: number | null;
  confidence: number | null;
}

// PCM16 to WAV encoder
function encodeWav(samples: Int16Array, sampleRate: number): ArrayBuffer {
  const byteLength = samples.length * 2;
  const buffer = new ArrayBuffer(44 + byteLength);
  const view = new DataView(buffer);
  const writeStr = (offset: number, s: string) => {
    for (let i = 0; i < s.length; i++) view.setUint8(offset + i, s.charCodeAt(i));
  };
  writeStr(0, "RIFF");
  view.setUint32(4, 36 + byteLength, true);
  writeStr(8, "WAVE");
  writeStr(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeStr(36, "data");
  view.setUint32(40, byteLength, true);
  const samplesView = new Uint8Array(buffer, 44);
  const rawView = new Uint8Array(samples.buffer, samples.byteOffset, samples.byteLength);
  samplesView.set(rawView);
  return buffer;
}

export default function RealTimePage() {
  const { t } = useI18n();
  const [recording, setRecording] = useState(false);
  const [segments, setSegments] = useState<Segment[]>([]);
  const [liveText, setLiveText] = useState("");
  const [speechDetected, setSpeechDetected] = useState(false);
  const [elapsed, setElapsed] = useState(0);
  const [statusMsg, setStatusMsg] = useState("");

  const speechBufferRef = useRef<Int16Array[]>([]);
  const elapsedRef = useRef(0);
  const isSpeakingRef = useRef(false);
  const silenceFramesRef = useRef(0);
  const timerRef = useRef<ReturnType<typeof setInterval>>(0 as unknown as ReturnType<typeof setInterval>);
  const streamRef = useRef<{ audioCtx: AudioContext; processor: ScriptProcessorNode; stream: MediaStream } | null>(null);

  const SAMPLE_RATE = 16000;
  const SILENCE_THRESHOLD = 50;
  const MIN_SPEECH_FRAMES = 15;
  const MIN_SILENCE_FRAMES = 25;

  // Listen for SSE stream events
  useEffect(() => {
    const unlisten = listen<StreamEvent>("stream-event", (event) => {
      const e = event.payload;
      if (e.event_type === "partial") {
        setLiveText(e.text);
      } else if (e.event_type === "final") {
        setSegments((prev) => [
          ...prev,
          { text: e.text, start: e.start ?? 0, end: e.end ?? 0, confidence: e.confidence ?? 0.9 },
        ]);
        setLiveText("");
      } else if (e.event_type === "error") {
        setStatusMsg(e.text);
        setLiveText("");
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  // Timer
  useEffect(() => {
    if (recording) {
      timerRef.current = setInterval(() => {
        elapsedRef.current += 1;
        setElapsed(elapsedRef.current);
      }, 1000);
    } else {
      if (timerRef.current) clearInterval(timerRef.current);
      elapsedRef.current = 0;
    }
    return () => { if (timerRef.current) clearInterval(timerRef.current); };
  }, [recording]);

  // RMS calculation for VAD
  const rms = (samples: Int16Array): number => {
    let sum = 0;
    for (let i = 0; i < samples.length; i++) sum += samples[i] * samples[i];
    return Math.sqrt(sum / samples.length);
  };

  const flushSegmentFn = async (): Promise<void> => {
    if (speechBufferRef.current.length < MIN_SPEECH_FRAMES) {
      speechBufferRef.current = [];
      isSpeakingRef.current = false;
      setSpeechDetected(false);
      return;
    }

    const totalLen = speechBufferRef.current.reduce((a, b) => a + b.length, 0);
    const combined = new Int16Array(totalLen);
    let offset = 0;
    for (const buf of speechBufferRef.current) {
      combined.set(buf, offset);
      offset += buf.length;
    }

    const wav = encodeWav(combined, SAMPLE_RATE);
    const wavBytes = Array.from(new Uint8Array(wav));

    speechBufferRef.current = [];
    silenceFramesRef.current = 0;
    isSpeakingRef.current = false;
    setSpeechDetected(false);
    setStatusMsg("识别中...");

    try {
      await invokeCmd("stream_transcribe", {
        wavBytes,
        modelId: "Qwen/Qwen3-ASR-0.6B-GGUF",
        language: "zh",
      });
      setStatusMsg("");
    } catch (e) {
      setStatusMsg(`识别失败: ${e}`);
    }
  };

  const processAudio = (samples: Int16Array): void => {
    const energy = rms(samples);
    const isSpeech = energy > SILENCE_THRESHOLD;

    if (isSpeech) {
      if (!isSpeakingRef.current) {
        isSpeakingRef.current = true;
        setSpeechDetected(true);
      }
      silenceFramesRef.current = 0;
      speechBufferRef.current.push(samples);
    } else if (isSpeakingRef.current) {
      silenceFramesRef.current++;
      speechBufferRef.current.push(samples);

      if (silenceFramesRef.current > MIN_SILENCE_FRAMES && speechBufferRef.current.length > MIN_SPEECH_FRAMES) {
        void flushSegmentFn();
      }
    }
  };

  const startMic = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: { sampleRate: SAMPLE_RATE, channelCount: 1, echoCancellation: true, noiseSuppression: true },
      });

      const ctx = new AudioContext({ sampleRate: SAMPLE_RATE });
      const source = ctx.createMediaStreamSource(stream);
      const processor = ctx.createScriptProcessor(1024, 1, 1);

      source.connect(processor);
      processor.connect(ctx.destination);

      processor.onaudioprocess = (e) => {
        const input = e.inputBuffer.getChannelData(0);
        const int16 = new Int16Array(input.length);
        for (let i = 0; i < input.length; i++) {
          int16[i] = Math.max(-32768, Math.min(32767, Math.round(input[i] * 32767)));
        }
        processAudio(int16);
      };

      streamRef.current = { audioCtx: ctx, processor, stream };

      setRecording(true);
      setElapsed(0);
      setSegments([]);
      setLiveText("");
      setStatusMsg("");
    } catch (e) {
      setStatusMsg(`麦克风访问失败: ${e}`);
    }
  };

  const stopMic = () => {
    if (isSpeakingRef.current) {
      void flushSegmentFn();
    }
    if (streamRef.current) {
      streamRef.current.processor.disconnect();
      streamRef.current.stream.getTracks().forEach((t) => t.stop());
      void streamRef.current.audioCtx.close();
      streamRef.current = null;
    }
    setRecording(false);
    setSpeechDetected(false);
  };

  const copyAll = () => {
    const text = segments.map((s) => s.text).join("");
    navigator.clipboard.writeText(text);
  };

  const formatTime = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-bold">{t("nav.realtime")}</h2>

      <div className="flex items-center gap-4">
        <button
          onClick={recording ? stopMic : startMic}
          className={`rounded-full px-8 py-4 text-lg font-bold transition-all ${
            recording
              ? "bg-destructive text-destructive-foreground animate-pulse"
              : "bg-primary text-primary-foreground hover:opacity-90"
          }`}
        >
          {recording ? `⏹ ${t("common.stop")}` : `🎙️ ${t("common.start")}`}
        </button>
        {recording && (
          <div className="flex items-center gap-3">
            <span>{formatTime(elapsed)}</span>
            <span className={`h-3 w-3 rounded-full ${speechDetected ? "bg-green-500 animate-ping" : "bg-muted-foreground"}`} />
            <span className="text-xs text-muted-foreground">{speechDetected ? "说话中" : "聆听中"}</span>
            {statusMsg && <span className="text-xs text-muted-foreground">{statusMsg}</span>}
          </div>
        )}
      </div>

      {/* Live transcript */}
      {recording && (
        <div className="rounded-xl border border-primary/30 bg-accent/30 p-4 min-h-16">
          <p className="text-lg">{liveText || "等待语音..."}</p>
        </div>
      )}

      {/* Segments */}
      {segments.length > 0 && (
        <div className="space-y-1">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium">
              识别结果 ({segments.length} 段)
            </h3>
            <button onClick={copyAll} className="rounded border border-border px-3 py-1 text-xs hover:bg-accent">
              {t("common.copy")}
            </button>
          </div>
          {segments.map((seg, i) => (
            <div key={i} className="rounded-lg border border-border px-3 py-2 text-sm">
              <span className="text-xs text-muted-foreground mr-2">
                {seg.start.toFixed(1)}s - {seg.end.toFixed(1)}s
              </span>
              {seg.text}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
