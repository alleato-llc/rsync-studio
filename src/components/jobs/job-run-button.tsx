import { Button } from "@/components/ui/button";
import { Play, Square, Loader2 } from "lucide-react";

interface JobRunButtonProps {
  isRunning: boolean;
  disabled?: boolean;
  hidden?: boolean;
  onRun: () => void;
  onCancel: () => void;
}

export function JobRunButton({ isRunning, disabled, hidden, onRun, onCancel }: JobRunButtonProps) {
  if (hidden && !isRunning) {
    return null;
  }

  if (isRunning) {
    return (
      <Button
        variant="ghost"
        size="icon"
        className="h-8 w-8 text-destructive"
        onClick={(e) => {
          e.stopPropagation();
          onCancel();
        }}
        title="Cancel job"
      >
        <Loader2 className="h-4 w-4 animate-spin absolute" />
        <Square className="h-3 w-3" />
      </Button>
    );
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      className="h-8 w-8 text-green-600"
      disabled={disabled}
      onClick={(e) => {
        e.stopPropagation();
        onRun();
      }}
      title="Run job"
    >
      <Play className="h-4 w-4" />
    </Button>
  );
}
