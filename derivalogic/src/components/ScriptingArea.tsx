
import React from 'react';
import { Textarea } from "@/components/ui/textarea";

interface ScriptingAreaProps {
  onScriptChange: (value: string) => void;
  value?: string;
  readOnly?: boolean;
}

export default function ScriptingArea({ onScriptChange, value, readOnly = false }: ScriptingAreaProps) {
  return (
    <div className="w-full">
      <Textarea
        placeholder="if EUR > 1.1 {
  Buy 1.000 EUR;
  Pay 2.000 CLP;
}"
        value={value || ''}
        onChange={(e) => onScriptChange(e.target.value)}
        readOnly={readOnly}
        rows={4}
        className="w-full resize-none font-mono bg-gray-50 border-gray-300 text-gray-900 placeholder:text-gray-500 focus:border-blue-500"
      />
    </div>
  );
}
