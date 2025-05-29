
import React from 'react';
import { SidebarProvider } from "@/components/ui/sidebar";
import { AppSidebar } from "@/components/AppSidebar";
import { TopBar } from "@/components/TopBar";
import { useDraftDeletion } from '@/hooks/useDraftDeletion';
import { useSidebarSave } from '@/hooks/useSidebarSave';
import { UnsavedChangesModal } from '@/components/UnsavedChangesModal';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

interface MainLayoutProps {
  children: React.ReactNode;
  scripts?: Script[];
  currentScript?: Script;
  onScriptSelect?: (script: Script) => void;
  onNewScript?: () => void;
  hasUnsavedChanges?: boolean;
  currentScriptName?: string;
  onSaveCurrentScript?: (name: string) => void;
}

export default function MainLayout({
  children,
  scripts = [],
  currentScript,
  onScriptSelect,
  onNewScript,
  hasUnsavedChanges = false,
  currentScriptName,
  onSaveCurrentScript
}: MainLayoutProps) {
  const {
    showUnsavedChangesModal,
    handleDeleteDraft,
    confirmDeleteWithUnsavedChanges,
    cancelDeleteWithUnsavedChanges
  } = useDraftDeletion(currentScript);

  const { handleSaveScript: saveDraft } = useSidebarSave();

  const handleDeleteDraftScript = (script: Script) => {
    handleDeleteDraft(script, hasUnsavedChanges && script.id === currentScript?.id);
  };

  const handleSaveSidebar = (script: Script) => {
    if (script.id === currentScript?.id && onSaveCurrentScript) {
      onSaveCurrentScript(currentScriptName || script.name);
    } else {
      saveDraft(script, script.name);
    }
  };

  return (
    <>
      <SidebarProvider>
        <AppSidebar 
          scripts={scripts}
          currentScript={currentScript}
          onScriptSelect={onScriptSelect}
          onNewScript={onNewScript}
          onDeleteDraft={handleDeleteDraftScript}
          onSaveScript={handleSaveSidebar}
        />
        <main className="flex-1 flex flex-col min-h-screen bg-gray-50">
          <TopBar />
          <div className="flex-1">
            <div className="container mx-auto p-6 bg-white shadow-lg min-h-screen">
              {children}
            </div>
          </div>
        </main>
      </SidebarProvider>

      <UnsavedChangesModal
        open={showUnsavedChangesModal}
        onConfirm={confirmDeleteWithUnsavedChanges}
        onCancel={cancelDeleteWithUnsavedChanges}
      />
    </>
  );
}
