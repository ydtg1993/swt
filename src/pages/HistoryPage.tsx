import { useEffect, useState, useRef } from "react";
import { useI18n } from "@/hooks/useI18n";
import { getHistory, searchHistory, getHistoryCount, deleteHistory, type HistoryRecord } from "@/hooks/useInvoke";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

const SOURCE_LABELS: Record<string, string> = {
  file: "📁 文件", mic: "🎙️ 录音", tts: "🔊 TTS", conversation: "💬 对话",
};

export default function HistoryPage() {
  const { t } = useI18n();
  const [records, setRecords] = useState<HistoryRecord[]>([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(0);
  const [query, setQuery] = useState("");
  const [debouncedQuery, setDebouncedQuery] = useState("");
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const PAGE_SIZE = 20;

  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => { setDebouncedQuery(query); setPage(0); }, 350);
    return () => { if (timerRef.current) clearTimeout(timerRef.current); };
  }, [query]);

  const load = async () => {
    const [data, count] = await Promise.all([
      debouncedQuery ? searchHistory(debouncedQuery, PAGE_SIZE) : getHistory(PAGE_SIZE, page * PAGE_SIZE),
      getHistoryCount(),
    ]);
    setRecords(data);
    setTotal(count);
  };

  useEffect(() => { load(); }, [page, debouncedQuery]);

  const handleDelete = async (id: number) => { await deleteHistory(id); load(); };

  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-extrabold uppercase">{t("nav.history")}</h2>
        <span className="text-sm font-bold text-muted-foreground bg-muted px-3 py-1 border-2 border-black dark:border-white">共 {total} 条</span>
      </div>

      <Input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="🔍 搜索文件名或内容..."
        className="max-w-md font-bold"
      />

      {records.length === 0 ? (
        <Card className="border-2 border-black dark:border-white">
          <CardContent className="py-12 text-center">
            <p className="text-4xl mb-2">📋</p>
            <p className="font-bold text-muted-foreground">暂无记录</p>
            <p className="text-xs text-muted-foreground mt-1">完成转写或 TTS 后将自动记录</p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-2">
          {records.map((r) => (
            <Card key={r.id} className="border-2 border-black dark:border-white hover:shadow-[3px_3px_0px_#000] dark:hover:shadow-[3px_3px_0px_#fff] transition-shadow">
              <CardContent className="py-3">
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 text-sm font-bold">
                      <span>{SOURCE_LABELS[r.source] ?? r.source}</span>
                      <span className="text-muted-foreground truncate max-w-[300px]">{r.file_name || "(无标题)"}</span>
                      {r.file_duration > 0 && <span className="text-xs">{Math.round(r.file_duration)}s</span>}
                    </div>
                    <p className="mt-1 text-sm line-clamp-3 whitespace-pre-wrap">{r.full_text || "(空)"}</p>
                    <div className="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-xs font-bold text-muted-foreground">
                      <span>{r.model_name}</span>
                      <span>{r.language}</span>
                      <span>{r.word_count} 字</span>
                      <span>{r.created_at}</span>
                    </div>
                  </div>
                  <Button size="sm" variant="destructive" onClick={() => handleDelete(r.id)}
                    className="shrink-0 ml-3 font-bold border-2 border-black dark:border-white text-xs">
                    {t("common.delete")}
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}

          {/* Pagination */}
          <div className="flex justify-center items-center gap-2 pt-4">
            <Button size="sm" variant="outline" disabled={page === 0} onClick={() => setPage(page - 1)}
              className="font-bold border-2 border-black dark:border-white text-xs">
              ← 上一页
            </Button>
            <span className="px-3 py-1 text-sm font-bold border-2 border-black dark:border-white bg-card">
              {page + 1} / {totalPages}
            </span>
            <Button size="sm" variant="outline" disabled={(page + 1) * PAGE_SIZE >= total}
              onClick={() => setPage(page + 1)}
              className="font-bold border-2 border-black dark:border-white text-xs">
              下一页 →
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
