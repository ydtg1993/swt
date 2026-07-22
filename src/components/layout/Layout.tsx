import { type ReactNode, useState, useEffect, useRef } from "react";
import Sidebar from "./Sidebar";
import TopBar from "./TopBar";
import { invokeCmd } from "@/hooks/useInvoke";

interface ServerStatus {
  running: boolean;
  backend?: string;
  models?: string[];
}

export default function Layout({ children }: { children: ReactNode }) {
  const [server, setServer] = useState<ServerStatus>({ running: false });
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    const poll = async () => {
      try {
        const data = await invokeCmd<{ backend?: string; models?: string[] }>("health_check");
        if (mountedRef.current) {
          setServer({ running: true, backend: data.backend, models: data.models });
        }
      } catch {
        if (mountedRef.current) {
          setServer({ running: false });
        }
      }
    };
    poll();
    const interval = setInterval(poll, 5000);
    return () => {
      mountedRef.current = false;
      clearInterval(interval);
    };
  }, []);

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <TopBar server={server} />
        <main className="flex-1 overflow-y-auto p-4">{children}</main>
      </div>
    </div>
  );
}
