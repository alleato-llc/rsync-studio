import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

export type NavPage = "jobs" | "history" | "statistics" | "tools" | "settings";

interface SidebarProps {
  currentPage: NavPage;
  onNavigate: (page: NavPage) => void;
}

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <aside className="w-56 border-r bg-muted/40 flex flex-col h-screen">
      <div className="p-4">
        <h1 className="text-lg font-semibold">Rsync Studio</h1>
        <p className="text-xs text-muted-foreground">Backup management</p>
      </div>
      <Separator />
      <nav className="flex-1 flex flex-col gap-1 p-2">
        <Button
          variant={currentPage === "jobs" ? "secondary" : "ghost"}
          className="justify-start"
          onClick={() => onNavigate("jobs")}
        >
          Jobs
        </Button>
        <Button
          variant={currentPage === "history" ? "secondary" : "ghost"}
          className="justify-start"
          onClick={() => onNavigate("history")}
        >
          History
        </Button>
        <Button
          variant={currentPage === "statistics" ? "secondary" : "ghost"}
          className="justify-start"
          onClick={() => onNavigate("statistics")}
        >
          Statistics
        </Button>
        <Button
          variant={currentPage === "tools" ? "secondary" : "ghost"}
          className="justify-start"
          onClick={() => onNavigate("tools")}
        >
          Tools
        </Button>
      </nav>
      <Separator />
      <div className="p-2">
        <Button
          variant={currentPage === "settings" ? "secondary" : "ghost"}
          className="justify-start w-full"
          onClick={() => onNavigate("settings")}
        >
          Settings
        </Button>
      </div>
    </aside>
  );
}
