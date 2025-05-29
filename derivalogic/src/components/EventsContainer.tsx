
import React from 'react';
import MainLayout from '@/components/MainLayout';
import { EventsHeader } from '@/components/EventsHeader';
import { EventsContent } from '@/components/EventsContent';
import { ScriptSaveModal } from '@/components/ScriptSaveModal';
import { UnsavedChangesModal } from '@/components/UnsavedChangesModal';
import { useEventsHandlers } from '@/hooks/useEventsHandlers';
import { useEventsNavigation } from '@/hooks/useEventsNavigation';

interface Event {
  id: string;
  eventDate: Date;
  script: string;
  name?: string;
  description?: string;
}

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: Event[];
}

interface EventsContainerProps {
  script: Script;
  scripts: Script[];
  events: Event[];
  isLoading: boolean;
  mutations: {
    saveScriptMutation: any;
    createScriptMutation: any;
    createEventMutation: any;
    updateEventMutation: any;
    deleteEventMutation: any;
    deleteDraftScriptMutation: any;
  };
  state: {
    hasUnsavedChanges: boolean;
    isSaveModalOpen: boolean;
    setIsSaveModalOpen: (open: boolean) => void;
    showUnsavedChangesModal: boolean;
    setShowUnsavedChangesModal: (show: boolean) => void;
    pendingNavigation: (() => void) | null;
    setPendingNavigation: (nav: (() => void) | null) => void;
    isModifyMode: boolean;
    setIsModifyMode: () => void;
    showSavedMessage: boolean;
    setShowSavedMessage: (show: boolean) => void;
    currentScriptName: string;
    setCurrentScriptName: (name: string) => void;
    isDraftWithChanges: () => boolean;
  };
  onConfirmUnsavedChanges: () => void;
  onCancelUnsavedChanges: () => void;
}

export function EventsContainer({
  script,
  scripts,
  events,
  isLoading,
  mutations,
  state,
  onConfirmUnsavedChanges,
  onCancelUnsavedChanges
}: EventsContainerProps) {
  const {
    saveScriptMutation,
    createScriptMutation,
    createEventMutation,
    updateEventMutation,
    deleteEventMutation
  } = mutations;

  const {
    hasUnsavedChanges,
    isSaveModalOpen,
    setIsSaveModalOpen,
    isModifyMode,
    setIsModifyMode,
    showSavedMessage,
    setShowSavedMessage,
    currentScriptName,
    setCurrentScriptName,
    isDraftWithChanges
  } = state;

  const eventHandlers = useEventsHandlers({
    isModifyMode,
    createEventMutation,
    updateEventMutation,
    deleteEventMutation,
    saveScriptMutation,
    setIsSaveModalOpen,
    setIsModifyMode,
    setShowSavedMessage,
    setCurrentScriptName,
    script
  });

  const navigationHandlers = useEventsNavigation({
    hasUnsavedChanges,
    isDraftWithChanges,
    setPendingNavigation: state.setPendingNavigation,
    setShowUnsavedChangesModal: state.setShowUnsavedChangesModal,
    createScriptMutation,
    script
  });

  const currentScriptData = scripts.find(s => s.id === script?.id);
  const scriptStatus = currentScriptData?.status || 'DRAFT';

  if (isLoading) {
    return (
      <MainLayout
        scripts={scripts}
        currentScript={script}
        currentScriptName={currentScriptName}
        onSaveCurrentScript={eventHandlers.handleSave}
        onScriptSelect={navigationHandlers.handleScriptSelect}
        onNewScript={navigationHandlers.handleNewScript}
        hasUnsavedChanges={hasUnsavedChanges}
      >
        <div className="flex items-center justify-center">
          <div>Loading events...</div>
        </div>
      </MainLayout>
    );
  }

  return (
    <>
      <MainLayout
        scripts={scripts}
        currentScript={script}
        currentScriptName={currentScriptName}
        onSaveCurrentScript={eventHandlers.handleSave}
        onScriptSelect={navigationHandlers.handleScriptSelect}
        onNewScript={navigationHandlers.handleNewScript}
        hasUnsavedChanges={hasUnsavedChanges}
      >
        <EventsHeader
          currentScriptName={currentScriptName}
          scriptStatus={scriptStatus}
          showSavedMessage={showSavedMessage}
          isModifyMode={isModifyMode}
          scriptId={script.id}
          referenceDate={script.referenceDate}
          eventsCount={events.length}
          onSave={eventHandlers.handleSave}
          onModify={eventHandlers.handleModify}
          onScriptNameChange={eventHandlers.handleScriptNameChange}
        />

        <EventsContent
          events={events}
          isModifyMode={isModifyMode}
          isCreatingEvent={createEventMutation.isPending}
          onAddEvent={eventHandlers.handleAddEvent}
          onRun={eventHandlers.handleRun}
          onScriptChange={eventHandlers.handleScriptChange}
          onDateChange={eventHandlers.handleDateChange}
          onNameChange={eventHandlers.handleNameChange}
          onDescriptionChange={eventHandlers.handleDescriptionChange}
          onDeleteEvent={eventHandlers.handleDeleteEvent}
        />
      </MainLayout>

      <ScriptSaveModal
        open={isSaveModalOpen}
        onClose={() => setIsSaveModalOpen(false)}
        onSave={eventHandlers.handleSaveScript}
        initialName={currentScriptName}
      />

      <UnsavedChangesModal
        open={state.showUnsavedChangesModal}
        onConfirm={onConfirmUnsavedChanges}
        onCancel={onCancelUnsavedChanges}
      />
    </>
  );
}
