import { useState, useEffect } from "react";
import {
  Briefcase,
  History,
  BarChart3,
  Hammer,
  Settings,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";

export type NavPage = "jobs" | "history" | "statistics" | "tools" | "settings";

interface SidebarProps {
  currentPage: NavPage;
  onNavigate: (page: NavPage) => void;
}

const STORAGE_KEY = "rsync-studio-sidebar-collapsed";

const navItems: { page: NavPage; label: string; icon: typeof Briefcase }[] = [
  { page: "jobs", label: "Jobs", icon: Briefcase },
  { page: "history", label: "History", icon: History },
  { page: "statistics", label: "Statistics", icon: BarChart3 },
  { page: "tools", label: "Tools", icon: Hammer },
];

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  const [collapsed, setCollapsed] = useState(() => {
    return localStorage.getItem(STORAGE_KEY) === "true";
  });

  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, String(collapsed));
  }, [collapsed]);

  return (
    <aside
      className={`${
        collapsed ? "w-14" : "w-56"
      } border-r bg-muted/40 flex flex-col h-screen sticky top-0 transition-all duration-200 overflow-hidden`}
    >
      <div className="p-4 overflow-hidden whitespace-nowrap">
        {collapsed ? (
          <div className="flex items-center justify-center">
            <span className="text-lg font-semibold">R</span>
          </div>
        ) : (
          <>
            <h1 className="text-lg font-semibold">Rsync Studio</h1>
            <p className="text-xs text-muted-foreground">Backup management</p>
          </>
        )}
      </div>
      <Separator />
      <nav className="flex-1 flex flex-col gap-1 p-2">
        {navItems.map(({ page, label, icon: Icon }) => (
          <Button
            key={page}
            variant={currentPage === page ? "secondary" : "ghost"}
            className={`${collapsed ? "justify-center px-0" : "justify-start"}`}
            onClick={() => onNavigate(page)}
            title={collapsed ? label : undefined}
          >
            {collapsed ? (
              <Icon className="h-4 w-4" />
            ) : (
              <>
                <Icon className="h-4 w-4 mr-2 shrink-0" />
                <span className="overflow-hidden whitespace-nowrap">{label}</span>
              </>
            )}
          </Button>
        ))}
      </nav>
      <Separator />
      <div className="p-2 space-y-1">
        <Button
          variant={currentPage === "settings" ? "secondary" : "ghost"}
          className={`w-full ${collapsed ? "justify-center px-0" : "justify-start"}`}
          onClick={() => onNavigate("settings")}
          title={collapsed ? "Settings" : undefined}
        >
          {collapsed ? (
            <Settings className="h-4 w-4" />
          ) : (
            <>
              <Settings className="h-4 w-4 mr-2 shrink-0" />
              <span className="overflow-hidden whitespace-nowrap">Settings</span>
            </>
          )}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className={`w-full ${collapsed ? "justify-center px-0" : "justify-start"}`}
          onClick={() => setCollapsed(!collapsed)}
          title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {collapsed ? (
            <ChevronRight className="h-4 w-4" />
          ) : (
            <>
              <ChevronLeft className="h-4 w-4 mr-2 shrink-0" />
              <span className="overflow-hidden whitespace-nowrap">Collapse</span>
            </>
          )}
        </Button>
      </div>
    </aside>
  );
}
