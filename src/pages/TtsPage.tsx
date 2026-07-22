import { useState, useEffect } from "react";
import { useI18n } from "@/hooks/useI18n";
import { invokeCmd, getConfig, listModels, type ModelInfo } from "@/hooks/useInvoke";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export default function TtsPage() {
  const { t } = useI18n();
  const [text, setText] = useState("");
  const [speed, setSpeed] = useState(1.0);
  const [voice, setVoice] = useState("default");
  const [modelId, setModelId] = useState("Qwen/Qwen3-TTS-0.6B-GGUF");
  const [generating, setGenerating] = useState(false);
  const [audioPath, setAudioPath] = useState("");
  const [ttsModels, setTtsModels] = useState<ModelInfo[]>([]);
  const [msg, setMsg] = useState("");

  useEffect(() => {
    listModels("tts").then(setTtsModels).catch(console.error);
    getConfig().then((c) => {
      setSpeed(c.tts.speed);
      setVoice(c.tts.voice);
      setModelId(c.tts.default_model);
    }).catch(console.error);
  }, []);

  const synthesize = async () => {
    if (!text.trim()) return;
    setGenerating(true);
    setMsg("生成中...");
    try {
      const path = await invokeCmd<string>("synthesize", { text, modelId, voice, speed });
      setAudioPath(path);
      setMsg("");
    } catch (e) { setMsg(`失败: ${e}`); }
    setGenerating(false);
  };

  const downloadedModels = ttsModels.filter(m => m.downloaded);

  return (
    <div className="space-y-4 max-w-2xl">
      <h2 className="text-xl font-extrabold uppercase">{t("nav.tts")}</h2>

      <Card className="border-2 border-black dark:border-white">
        <CardHeader className="pb-2">
          <CardTitle className="text-base font-extrabold">配置</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-wrap gap-4 items-end">
          <div>
            <label className="text-xs font-bold uppercase block mb-1">模型</label>
            <select value={modelId} onChange={(e) => setModelId(e.target.value)}
              className="border-2 border-black dark:border-white bg-background px-3 py-2 text-sm font-bold rounded-lg w-56">
              {downloadedModels.length > 0 ? downloadedModels.map(m => (
                <option key={m.id} value={m.id}>{m.name}</option>
              )) : <option value="">请先下载 TTS 模型</option>}
            </select>
          </div>
          <div>
            <label className="text-xs font-bold uppercase block mb-1">语速</label>
            <div className="flex items-center gap-2">
              <input type="range" min="0.5" max="2.0" step="0.1" value={speed}
                onChange={(e) => setSpeed(parseFloat(e.target.value))}
                className="w-28 accent-primary" />
              <span className="text-xs font-bold w-10">{speed}x</span>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card className="border-2 border-black dark:border-white">
        <CardContent className="pt-4 space-y-3">
          <textarea value={text} onChange={(e) => setText(e.target.value)}
            rows={5} maxLength={4096}
            className="w-full border-2 border-black dark:border-white bg-background p-4 text-sm font-bold rounded-lg resize-none placeholder:text-muted-foreground"
            placeholder="输入要合成的文字（最多 4096 字符）..." />
          <div className="flex items-center justify-between">
            <Button onClick={synthesize} disabled={!text.trim() || generating}
              className="font-bold border-2 border-black dark:border-white shadow-[4px_4px_0px_#000] dark:shadow-[4px_4px_0px_#fff]">
              {generating ? "🎵 合成中..." : "🔊 生成语音"}
            </Button>
            <div className="text-xs font-bold text-muted-foreground">{text.length} / 4096</div>
          </div>
          {msg && <p className="text-sm font-bold text-muted-foreground">{msg}</p>}
        </CardContent>
      </Card>

      {audioPath && (
        <Card className="border-2 border-black dark:border-white">
          <CardContent className="pt-4">
            <audio controls className="w-full" src={`asset://localhost/${audioPath}`} />
          </CardContent>
        </Card>
      )}
    </div>
  );
}
