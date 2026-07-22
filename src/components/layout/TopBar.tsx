import { Badge } from "@/components/ui/badge";

interface ServerStatus { running: boolean; backend?: string; models?: string[]; }

export default function TopBar({ server }: { server: ServerStatus }) {
  return (
    <header className="flex h-11 items-center justify-between border-b-2 border-black dark:border-white bg-card px-4 shrink-0">
      <div className="flex items-center gap-2">
        <span className={`h-2.5 w-2.5 rounded-full border-2 border-black dark:border-white ${server.running ? "bg-green-400" : "bg-red-400"}`} />
        <span className="text-xs font-bold uppercase tracking-wide">
          {server.running ? `ONLINE (${server.backend ?? "?"})` : "OFFLINE"}
        </span>
      </div>
      {server.running && server.models && server.models.length > 0 && (
        <div className="flex gap-1">
          {server.models.map((m) => (
            <Badge key={m} variant="outline" className="border-2 border-black dark:border-white text-[10px] font-bold">{m}</Badge>
          ))}
        </div>
      )}
    </header>
  );
}
