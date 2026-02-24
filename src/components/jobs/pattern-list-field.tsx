import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { X, Plus } from "lucide-react";

interface PatternListFieldProps {
  label: string;
  patterns: string[];
  onChange: (patterns: string[]) => void;
  placeholder?: string;
}

export function PatternListField({
  label,
  patterns,
  onChange,
  placeholder = "e.g. *.tmp",
}: PatternListFieldProps) {
  const [input, setInput] = useState("");

  function handleAdd() {
    const trimmed = input.trim();
    if (trimmed && !patterns.includes(trimmed)) {
      onChange([...patterns, trimmed]);
      setInput("");
    }
  }

  function handleRemove(index: number) {
    onChange(patterns.filter((_, i) => i !== index));
  }

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    }
  }

  return (
    <div className="space-y-2">
      <Label>{label}</Label>
      <div className="flex gap-2">
        <Input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={placeholder}
          className="flex-1"
        />
        <Button type="button" variant="outline" size="icon" onClick={handleAdd}>
          <Plus className="h-4 w-4" />
        </Button>
      </div>
      {patterns.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {patterns.map((pattern, i) => (
            <span
              key={i}
              className="inline-flex items-center gap-1 rounded-md bg-secondary px-2 py-1 text-sm"
            >
              <code>{pattern}</code>
              <button
                type="button"
                onClick={() => handleRemove(i)}
                className="text-muted-foreground hover:text-foreground"
              >
                <X className="h-3 w-3" />
              </button>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
