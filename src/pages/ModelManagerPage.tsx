import { useEffect, useState, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "@/hooks/useI18n";
import { listModels, startDownload, cancelDownload, deleteModelCmd, type ModelInfo, type DownloadProgress } from "@/hooks/useInvoke";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";

const CATEGORIES = [
  { key: "asr_cpp", label: "语音识别", icon: "🎙️" },
  { key: "tts", label: "语音合成", icon: "🔊" },
  { key: "vad", label: "语音检测", icon: "🔇" },
  { key: "diar", label: "说话人分离", icon: "👥" },
  { key: "align", label: "强制对齐", icon: "📏" },
  { key: "vc", label: "语音转换", icon: "🔄" },
  { key: "codec", label: "音频编解码", icon: "📦" },
  { key: "music", label: "音乐生成", icon: "🎵" },
  { key: "sep", label: "音源分离", icon: "✂️" },
];

function fmtBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function ModelManagerPage() {
  const { t } = useI18n();
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [activeDownload, setActiveDownload] = useState<string | null>(null);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [filter, setFilter] = useState("all");
  const [search, setSearch] = useState("");

  const load = async () => { const all = await listModels(); setModels(all); };
  useEffect(() => { load(); }, []);

  useEffect(() => {
    const u = listen<DownloadProgress>("download-progress", (e) => {
      setProgress((p) => ({ ...p, [e.payload.model_id]: e.payload }));
      if (e.payload.state === "complete" || e.payload.state === "error") {
        setActiveDownload(null);
        load();
      }
    });
    return () => { u.then((fn) => fn()); };
  }, []);

  const handleDownload = async (id: string) => {
    setActiveDownload(id);
    setProgress((p) => ({ ...p, [id]: { model_id: id, downloaded_bytes: 0, total_bytes: 0, percentage: 0, state: "downloading" } }));
    try { await startDownload(id); } catch { setActiveDownload(null); }
  };

  const handleDelete = async (id: string) => { await deleteModelCmd(id); load(); };

  // Compute category counts
  const catCounts = useMemo(() => {
    const counts: Record<string, number> = { all: models.length };
    for (const cat of CATEGORIES) {
      counts[cat.key] = models.filter((m) => m.category === cat.key).length;
    }
    return counts;
  }, [models]);

  // Filter + search
  const filtered = useMemo(() => {
    let list = filter === "all" ? models : models.filter((m) => m.category === filter);
    if (search.trim()) {
      const q = search.toLowerCase();
      list = list.filter((m) =>
        m.name.toLowerCase().includes(q) ||
        m.description.toLowerCase().includes(q) ||
        m.model_family.toLowerCase().includes(q)
      );
    }
    return list;
  }, [models, filter, search]);

  const ModelCard = ({ m }: { m: ModelInfo }) => {
    const p = progress[m.id];
    const isActive = activeDownload === m.id;
    const canDownload = m.gguf_repo_id !== "";

    return (
      <Card className="border-2 border-black dark:border-white hover:shadow-[3px_3px_0px_#000] dark:hover:shadow-[3px_3px_0px_#fff] transition-shadow">
        <CardContent className="p-4 space-y-3">
          {/* Header */}
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0 flex-1">
              <h3 className="text-sm font-extrabold truncate">{m.name}</h3>
              <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">{m.description}</p>
            </div>
            <div className="shrink-0 flex flex-col items-end gap-1">
              {m.downloaded && (
                <Badge className="border-2 border-black dark:border-white bg-green-300 dark:bg-green-700 font-bold text-[10px]">✓ 已下载</Badge>
              )}
              {!canDownload && !m.downloaded && (
                <Badge className="border-2 border-black dark:border-white bg-yellow-300 dark:bg-yellow-700 font-bold text-[10px]">需编译</Badge>
              )}
            </div>
          </div>

          {/* Tags */}
          <div className="flex flex-wrap gap-1.5">
            <Badge variant="outline" className="text-[10px] font-bold border-2 border-black dark:border-white">
              📦 {m.estimated_size}
            </Badge>
            <Badge variant="outline" className="text-[10px] font-bold border-2 border-black dark:border-white">
              💻 {m.cpu_suitable}
            </Badge>
            <Badge variant="outline" className="text-[10px] font-bold border-2 border-black dark:border-white">
              {m.languages.slice(0, 4).join("/")}{m.languages.length > 4 ? "…" : ""}
            </Badge>
          </div>

          {/* Progress */}
          {(isActive || p) && (
            <div className="space-y-1">
              <Progress value={p?.percentage ?? 0} className="h-3 border-2 border-black dark:border-white" />
              <div className="flex justify-between text-[10px] font-bold text-muted-foreground">
                <span>{p?.state === "downloading" ? "下载中" : p?.state}</span>
                <span>
                  {p?.percentage.toFixed(1)}%
                  {p && p.downloaded_bytes > 0 && (
                    <> · {fmtBytes(p.downloaded_bytes)} / {p.total_bytes > 0 ? fmtBytes(p.total_bytes) : "?"}</>
                  )}
                </span>
              </div>
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2">
            {m.downloaded ? (
              <Button size="sm" variant="destructive" onClick={() => handleDelete(m.id)}
                className="font-bold border-2 border-black dark:border-white text-xs flex-1">
                🗑 删除
              </Button>
            ) : isActive ? (
              <Button size="sm" variant="outline" onClick={cancelDownload}
                className="font-bold border-2 border-black dark:border-white text-xs flex-1">
                ⏹ 取消下载
              </Button>
            ) : canDownload ? (
              <Button size="sm" onClick={() => handleDownload(m.id)} disabled={activeDownload !== null}
                className="font-bold border-2 border-black dark:border-white shadow-[3px_3px_0px_#000] dark:shadow-[3px_3px_0px_#fff] text-xs flex-1">
                ⬇ 下载
              </Button>
            ) : (
              <p className="text-[10px] text-muted-foreground italic self-center flex-1">
                需 audiocpp_server 编译
              </p>
            )}
          </div>
        </CardContent>
      </Card>
    );
  };

  return (
    <div className="flex gap-4 h-full">
      {/* Left sidebar — category filter */}
      <aside className="w-44 shrink-0 space-y-1">
        <h2 className="text-sm font-extrabold uppercase px-2 py-1">{t("nav.models")}</h2>
        <Separator className="border-black dark:border-white mb-1" />
        <button
          onClick={() => setFilter("all")}
          className={`w-full flex items-center gap-2 px-3 py-2 text-xs font-bold border-2 transition-all ${
            filter === "all"
              ? "bg-yellow-300 dark:bg-yellow-600 border-black dark:border-white shadow-[3px_3px_0px_#000] dark:shadow-[3px_3px_0px_#fff]"
              : "border-transparent hover:border-black dark:hover:border-white"
          }`}
        >
          <span>📋</span>
          <span className="flex-1 text-left">全部</span>
          <Badge className="border-2 border-black dark:border-white text-[10px] font-bold px-1.5">{catCounts.all}</Badge>
        </button>
        {CATEGORIES.map((cat) => (
          <button
            key={cat.key}
            onClick={() => setFilter(cat.key)}
            className={`w-full flex items-center gap-2 px-3 py-2 text-xs font-bold border-2 transition-all ${
              filter === cat.key
                ? "bg-yellow-300 dark:bg-yellow-600 border-black dark:border-white shadow-[3px_3px_0px_#000] dark:shadow-[3px_3px_0px_#fff]"
                : "border-transparent hover:border-black dark:hover:border-white"
            }`}
          >
            <span>{cat.icon}</span>
            <span className="flex-1 text-left">{cat.label}</span>
            <Badge className="border-2 border-black dark:border-white text-[10px] font-bold px-1.5 min-w-[20px] text-center">{catCounts[cat.key] ?? 0}</Badge>
          </button>
        ))}
      </aside>

      {/* Main area */}
      <div className="flex-1 min-w-0 space-y-3">
        {/* Search + stats */}
        <div className="flex items-center gap-3">
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="🔍 搜索模型..."
            className="max-w-xs font-bold text-sm"
          />
          <span className="text-xs font-bold text-muted-foreground">
            {filtered.length} 个模型
          </span>
        </div>

        {/* Grid */}
        {filtered.length === 0 ? (
          <Card className="border-2 border-black dark:border-white">
            <CardContent className="py-16 text-center">
              <p className="text-4xl mb-3">🧩</p>
              <p className="font-bold text-muted-foreground">
                {search ? "无匹配模型" : "此分类暂无模型"}
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                {search ? "尝试其他关键词" : "确保已安装 audiocpp_server 并下载模型"}
              </p>
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
            {filtered.map((m) => <ModelCard key={m.id} m={m} />)}
          </div>
        )}
      </div>
    </div>
  );
}
