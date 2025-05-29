
import React from 'react';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarGroupContent,
  SidebarMenu,
} from "@/components/ui/sidebar";
import { SidebarScriptItem } from './SidebarScriptItem';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

interface SidebarScriptsListProps {
  scripts: Script[];
  currentScript?: Script;
  openScripts: string[];
  onToggleScript: (scriptId: string) => void;
  onScriptSelect?: (script: Script) => void;
  onDeleteDraft?: (script: Script) => void;
  onSaveScript?: (script: Script) => void;
}

export function SidebarScriptsList({
  scripts,
  currentScript,
  openScripts,
  onToggleScript,
  onScriptSelect,
  onDeleteDraft,
  onSaveScript
}: SidebarScriptsListProps) {
  // Separate drafts and saved scripts
  const draftScripts = scripts.filter(script => script.status === 'DRAFT');
  const savedScripts = scripts.filter(script => script.status !== 'DRAFT');

  return (
    <>
      <SidebarGroup>
        <SidebarGroupLabel className="text-gray-700 font-medium">
          Drafts ({draftScripts.length})
        </SidebarGroupLabel>
        <SidebarGroupContent>
          {draftScripts.length === 0 ? (
            <p className="text-sm text-gray-500 p-2">No drafts found</p>
          ) : (
            <SidebarMenu>
              {draftScripts.map((script) => (
                <SidebarScriptItem
                  key={script.id}
                  script={script}
                  isDraft={true}
                  isOpen={openScripts.includes(script.id)}
                  currentScript={currentScript}
                  onToggle={() => onToggleScript(script.id)}
                  onScriptSelect={onScriptSelect}
                  onDeleteDraft={onDeleteDraft}
                  onSaveScript={onSaveScript}
                />
              ))}
            </SidebarMenu>
          )}
        </SidebarGroupContent>
      </SidebarGroup>

      <SidebarGroup>
        <SidebarGroupLabel className="text-gray-700 font-medium">
          My Scripts ({savedScripts.length})
        </SidebarGroupLabel>
        <SidebarGroupContent>
          {savedScripts.length === 0 ? (
            <p className="text-sm text-gray-500 p-2">No scripts found</p>
          ) : (
            <SidebarMenu>
              {savedScripts.map((script) => (
                <SidebarScriptItem
                  key={script.id}
                  script={script}
                  isDraft={false}
                  isOpen={openScripts.includes(script.id)}
                  currentScript={currentScript}
                  onToggle={() => onToggleScript(script.id)}
                  onScriptSelect={onScriptSelect}
                  onDeleteDraft={onDeleteDraft}
                  onSaveScript={onSaveScript}
                />
              ))}
            </SidebarMenu>
          )}
        </SidebarGroupContent>
      </SidebarGroup>
    </>
  );
}
