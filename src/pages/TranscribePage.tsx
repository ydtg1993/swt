import { useState, useEffect, useCallback, type DragEvent } from "react";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "@/hooks/useI18n";
import { invokeCmd, listModels, type ModelInfo } from "@/hooks/useInvoke";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface FileItem {
  path: string;
  name: string;
  status: "queued" | "processing" | "done" | "error";
  progress: number;
  message: string;
  result?: TranscribeResult;
}

interface TranscribeResult {
  file_path: string; file_name: string; file_duration: number; full_text: string;
  segments: Segment[]; model_name: string; language: string;
}
interface Segment { text: string; start: number; end: number; confidence: number; speaker: string; }
interface BatchProgress { file_index: number; total_files: number; file_name: string; percentage: number; state: string; message: string; }

export default function TranscribePage() {
  const { t } = useI18n();
  const [files, setFiles] = useState<FileItem[]>([]);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [modelId, setModelId] = useState("Qwen/Qwen3-ASR-0.6B-GGUF");
  const [language, setLanguage] = useState("zh");
  const [processing, setProcessing] = useState(false);
  const [selectedResult, setSelectedResult] = useState<number | null>(null);
  const [dragOver, setDragOver] = useState(false);

  useEffect(() => {
    listModels("asr_cpp").then(setModels).catch(console.error);
  }, []);

  useEffect(() => {
    const u1 = listen<BatchProgress>("batch-progress", (event) => {
      const p = event.payload;
      setFiles((prev) => prev.map((f, i) =>
        i === (p.file_index + (prev.length - p.total_files))
          ? { ...f, status: p.state as FileItem["status"], progress: p.percentage, message: p.message } : f));
    });
    const u2 = listen<number>("batch-all-done", () => setProcessing(false));
    return () => { u1.then((fn) => fn()); u2.then((fn) => fn()); };
  }, []);

  const handleDrop = useCallback((e: DragEvent) => {
    e.preventDefault(); setDragOver(false);
    const paths: string[] = [];
    if (e.dataTransfer.files) {
      for (let i = 0; i < e.dataTransfer.files.length; i++) {
        const f = e.dataTransfer.files[i] as File & { path?: string };
        if (f.path) paths.push(f.path);
      }
    }
    if (paths.length === 0) return;
    setFiles((prev) => [...prev, ...paths.map((p) => ({
      path: p, name: p.split(/[/\\]/).pop() || p, status: "queued" as const, progress: 0, message: "" }))]);
  }, []);

  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: "Audio/Video", extensions: ["mp3", "wav", "flac", "ogg", "m4a", "mp4", "mkv", "avi", "mov", "webm", "wmv"] }],
      });
      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      setFiles((prev) => [...prev, ...paths.map((p) => ({
        path: p, name: p.split(/[/\\]/).pop() || p, status: "queued" as const, progress: 0, message: "" }))]);
    } catch (e) { console.error("File dialog error:", e); }
  };

  const removeFile = (index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
    if (selectedResult === index) setSelectedResult(null);
  };

  const startTranscription = async () => {
    if (files.length === 0 || processing) return;
    setProcessing(true);
    const paths = files.map((f) => f.path);
    setFiles((prev) => prev.map((f) => ({ ...f, status: "queued" as const, progress: 0, message: "" })));
    try {
      const results = await invokeCmd<TranscribeResult[]>("transcribe_batch", { filePaths: paths, modelId, language });
      setFiles((prev) => prev.map((f) => {
        const r = results.find((res: TranscribeResult) => res.file_path === f.path);
        const isError = r && r.full_text.startsWith("[ERROR]");
        return { ...f, status: r && !isError ? "done" : "error", progress: 100, result: r };
      }));
    } catch (e) {
      setFiles((prev) => prev.map((f) => ({ ...f, status: "error" as const, message: String(e) })));
      setProcessing(false);
    }
  };

  const exportResult = (result: TranscribeResult, format: "txt" | "json" | "srt") => {
    let content = ""; const name = result.file_name.replace(/\.[^.]+$/, "");
    if (format === "txt") { content = result.full_text; }
    else if (format === "json") { content = JSON.stringify(result, null, 2); }
    else if (format === "srt") {
      content = result.segments.map((seg, i) => {
        const toTime = (s: number) => `${String(Math.floor(s/3600)).padStart(2,"0")}:${String(Math.floor((s%3600)/60)).padStart(2,"0")}:${String(Math.floor(s%60)).padStart(2,"0")},${String(Math.floor((s%1)*1000)).padStart(3,"0")}`;
        return `${i+1}\n${toTime(seg.start)} --> ${toTime(seg.end)}\n${seg.text}\n`; }).join("\n"); }
    const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob); const a = document.createElement("a");
    a.href = url; a.download = `${name}.${format}`; a.click(); URL.revokeObjectURL(url);
  };

  const result = selectedResult !== null ? files[selectedResult]?.result : null;

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-bold">{t("nav.transcribe")}</h2>

      {/* Config bar */}
      <Card>
        <CardContent className="flex flex-wrap gap-3 items-end pt-4">
          <div>
            <label className="text-xs font-bold uppercase">模型</label>
            <select value={modelId} onChange={(e) => setModelId(e.target.value)}
              className="block border-2 border-black dark:border-white bg-background px-3 py-2 text-sm font-bold w-56 rounded-lg">
              {models.filter(m => m.downloaded).length > 0
                ? models.filter(m => m.downloaded).map(m => <option key={m.id} value={m.id}>{m.name}</option>)
                : <option value="">⚠️ 无可用模型 — 请先到模型管理下载</option>}
            </select>
          </div>
          <div>
            <label className="text-xs font-bold uppercase">语言</label>
            <select value={language} onChange={(e) => setLanguage(e.target.value)}
              className="block border-2 border-black dark:border-white bg-background px-3 py-2 text-sm font-bold rounded-lg">
              <option value="zh">中文</option><option value="en">English</option>
              <option value="ja">日本語</option><option value="ko">한국어</option><option value="auto">auto</option>
            </select>
          </div>
          <Button onClick={startTranscription} disabled={files.length === 0 || processing}
            className="font-bold border-2 border-black dark:border-white shadow-[4px_4px_0px_#000] dark:shadow-[4px_4px_0px_#fff]">
            {processing ? "处理中..." : t("common.start")}
          </Button>
        </CardContent>
      </Card>

      {/* Drop zone */}
      <div
        onDrop={handleDrop}
        onDragOver={(e) => { e.preventDefault(); setDragOver(true); }}
        onDragLeave={() => setDragOver(false)}
        onClick={handleFileSelect}
        className={`border-2 border-dashed border-black dark:border-white p-10 text-center cursor-pointer transition-colors rounded-xl ${
          dragOver ? "bg-yellow-200 dark:bg-yellow-900" : "bg-muted/30 hover:bg-muted/50"}`}
      >
        <p className="text-4xl mb-2">📁</p>
        <p className="font-bold text-lg">拖放音频/视频文件，或点击选择</p>
        <p className="text-sm text-muted-foreground mt-1">MP3 / WAV / FLAC / MP4 / MKV …</p>
      </div>

      {/* File list */}
      {files.length > 0 && (
        <Card>
          <CardHeader className="pb-2"><CardTitle className="text-base">文件列表 ({files.length})</CardTitle></CardHeader>
          <CardContent className="space-y-1">
            {files.map((f, i) => (
            <div key={i} onClick={() => setSelectedResult(i)}
              className={`flex items-center gap-3 border-2 border-black dark:border-white px-3 py-2 cursor-pointer ${
                selectedResult === i ? "bg-yellow-200 dark:bg-yellow-900" : "hover:bg-muted/50"}`}>
              <span className="shrink-0 w-6 text-center">
                {f.status === "done" ? "✅" : f.status === "error" ? "❌" : f.status === "processing" ? "⏳" : "📄"}
              </span>
              <span className="flex-1 text-sm font-bold truncate">{f.name}</span>
              {f.status === "processing" && <Progress value={f.progress} className="w-32" />}
              {f.status === "processing" && <span className="text-xs w-16 text-right">{f.message}</span>}
              <button onClick={(e) => { e.stopPropagation(); removeFile(i); }}
                className="font-bold hover:text-red-600 text-xl leading-none">&times;</button>
            </div>
            ))}
          </CardContent>
        </Card>
      )}

      {/* Result viewer */}
      {result && (
        <Card className="border-2 border-black dark:border-white">
          <CardHeader className="pb-2">
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="text-base">{result.file_name}</CardTitle>
                <div className="flex gap-2 mt-1">
                  <Badge variant="outline" className="border-2 border-black dark:border-white">{result.model_name}</Badge>
                  <Badge variant="outline" className="border-2 border-black dark:border-white">{result.file_duration.toFixed(1)}s</Badge>
                  <Badge variant="outline" className="border-2 border-black dark:border-white">{result.language}</Badge>
                </div>
              </div>
              <div className="flex gap-1">
                {(["txt","json","srt"] as const).map((fmt) => (
                  <Button key={fmt} size="sm" onClick={() => exportResult(result, fmt)}
                    className="font-bold border-2 border-black dark:border-white text-xs">.{fmt}</Button>))}
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <Tabs defaultValue="text">
              <TabsList className="border-2 border-black dark:border-white">
                <TabsTrigger value="text" className="font-bold">全文</TabsTrigger>
                <TabsTrigger value="segments" className="font-bold">分段</TabsTrigger>
              </TabsList>
              <TabsContent value="text" className="mt-2">
                <div className="max-h-80 overflow-y-auto border-2 border-black dark:border-white p-4 bg-muted/20">
                  <pre className="text-sm whitespace-pre-wrap font-sans">{result.full_text}</pre>
                </div>
              </TabsContent>
              <TabsContent value="segments" className="mt-2">
                <div className="max-h-80 overflow-y-auto border-2 border-black dark:border-white text-xs">
                  <table className="w-full"><thead>
                    <tr className="text-left font-bold border-b-2 border-black dark:border-white">
                      <th className="py-1 px-2">#</th><th className="py-1 px-2">开始</th><th className="py-1 px-2">结束</th><th className="py-1 px-2">置信度</th><th className="py-1 px-2">文本</th>
                    </tr></thead><tbody>
                    {result.segments.map((seg, si) => (
                      <tr key={si} className="border-b border-black/20 dark:border-white/20">
                        <td className="py-1 px-2">{si+1}</td><td className="py-1 px-2">{seg.start.toFixed(1)}s</td>
                        <td className="py-1 px-2">{seg.end.toFixed(1)}s</td>
                        <td className="py-1 px-2">{(seg.confidence*100).toFixed(0)}%</td>
                        <td className="py-1 px-2">{seg.text}</td></tr>))}
                  </tbody></table>
                </div>
              </TabsContent>
            </Tabs>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
