import { useState, useEffect } from "react";
import { ExternalLink } from "lucide-react";
import { getVersion, getTauriVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

export function AboutPage() {
  const [appVersion, setAppVersion] = useState("");
  const [tauriVersion, setTauriVersion] = useState("");

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => setAppVersion("unknown"));
    getTauriVersion().then(setTauriVersion).catch(() => setTauriVersion("unknown"));
  }, []);

  const techStack = [
    { name: "Tauri", detail: tauriVersion ? `v${tauriVersion}` : "v2" },
    { name: "React", detail: "v19" },
    { name: "TypeScript", detail: null },
    { name: "Rust", detail: null },
    { name: "shadcn/ui", detail: null },
  ];

  return (
    <div className="flex items-start justify-center pt-16">
      <Card className="w-full max-w-md">
        <CardContent className="flex flex-col items-center gap-6 pt-8 pb-8">
          <img
            src="/icon.png"
            alt="Rsync Studio"
            className="h-24 w-24 rounded-2xl"
          />

          <div className="text-center">
            <h1 className="text-2xl font-bold">Rsync Studio</h1>
            {appVersion && (
              <Badge variant="secondary" className="mt-2">
                v{appVersion}
              </Badge>
            )}
          </div>

          <p className="text-sm text-muted-foreground text-center max-w-xs">
            A cross-platform desktop application for managing rsync backup jobs.
          </p>

          <Separator />

          <div className="w-full">
            <h2 className="text-sm font-medium mb-3 text-center">Tech Stack</h2>
            <div className="flex flex-wrap justify-center gap-2">
              {techStack.map(({ name, detail }) => (
                <Badge key={name} variant="outline">
                  {name}
                  {detail && (
                    <span className="ml-1 text-muted-foreground">{detail}</span>
                  )}
                </Badge>
              ))}
            </div>
          </div>

          <Separator />

          <Button
            variant="outline"
            size="sm"
            onClick={() => open("https://github.com/alleato-llc/rsync-studio")}
          >
            <ExternalLink className="h-4 w-4 mr-2" />
            View on GitHub
          </Button>

          <div className="text-center text-xs text-muted-foreground">
            <p>Built by Alleato LLC</p>
            <p className="mt-1">MIT License</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
