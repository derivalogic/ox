
import React, { useEffect, useRef } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { EventsContainer } from '@/components/EventsContainer';
import { useEventsData } from '@/hooks/useEventsData';
import { useEventsState } from '@/hooks/useEventsState';
import { useAuth } from '@/hooks/useAuth';

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

export function EventsPage() {
  const location = useLocation();
  const navigate = useNavigate();
  const script = location.state?.script as Script;
  const { user } = useAuth();
  const saveProcessedRef = useRef(false);

  const {
    scripts,
    events,
    isLoading,
    saveScriptMutation,
    createScriptMutation,
    createEventMutation,
    updateEventMutation,
    deleteEventMutation,
    deleteDraftScriptMutation
  } = useEventsData(script);

  const {
    hasUnsavedChanges,
    isSaveModalOpen,
    setIsSaveModalOpen,
    showUnsavedChangesModal,
    setShowUnsavedChangesModal,
    pendingNavigation,
    setPendingNavigation,
    isModifyMode,
    setIsModifyMode,
    showSavedMessage,
    setShowSavedMessage,
    currentScriptName,
    setCurrentScriptName,
    markUnsavedChanges,
    markSaved,
    isDraftWithChanges
  } = useEventsState(script);

  useEffect(() => {
    if (!script || !user) {
      navigate('/');
    }
  }, [script, user, navigate]);

  // Handle save script success - use ref to prevent infinite loop
  useEffect(() => {
    if (saveScriptMutation.isSuccess && saveScriptMutation.data && !saveProcessedRef.current) {
      console.log('Save mutation successful, calling markSaved with:', saveScriptMutation.data.scriptName);
      markSaved(saveScriptMutation.data.scriptName);
      saveProcessedRef.current = true;

      // Update the script in location state so other pages see the new name
      const updated = {
        ...script!,
        name: saveScriptMutation.data.scriptName,
        status: 'SAVED'
      };
      navigate('/events', { state: { script: updated }, replace: true });
    }
  }, [saveScriptMutation.isSuccess, saveScriptMutation.data, markSaved, script, navigate]);

  // Reset the ref when mutation is idle
  useEffect(() => {
    if (saveScriptMutation.isIdle) {
      saveProcessedRef.current = false;
    }
  }, [saveScriptMutation.isIdle]);

  // Handle mutation success for unsaved changes
  useEffect(() => {
    if (createEventMutation.isSuccess || updateEventMutation.isSuccess || deleteEventMutation.isSuccess) {
      markUnsavedChanges();
    }
  }, [createEventMutation.isSuccess, updateEventMutation.isSuccess, deleteEventMutation.isSuccess, markUnsavedChanges]);

  const handleConfirmUnsavedChanges = () => {
    setShowUnsavedChangesModal(false);
    
    if (pendingNavigation) {
      pendingNavigation();
      setPendingNavigation(null);
    }
  };

  const handleCancelUnsavedChanges = () => {
    setShowUnsavedChangesModal(false);
    setPendingNavigation(null);
  };

  if (!script || !user) {
    return (
      <div className="text-white">Loading...</div>
    );
  }

  return (
    <EventsContainer
      script={script}
      scripts={scripts}
      events={events}
      isLoading={isLoading}
      mutations={{
        saveScriptMutation,
        createScriptMutation,
        createEventMutation,
        updateEventMutation,
        deleteEventMutation,
        deleteDraftScriptMutation
      }}
      state={{
        hasUnsavedChanges,
        isSaveModalOpen,
        setIsSaveModalOpen,
        showUnsavedChangesModal,
        setShowUnsavedChangesModal,
        pendingNavigation,
        setPendingNavigation,
        isModifyMode,
        setIsModifyMode,
        showSavedMessage,
        setShowSavedMessage,
        currentScriptName,
        setCurrentScriptName,
        isDraftWithChanges
      }}
      onConfirmUnsavedChanges={handleConfirmUnsavedChanges}
      onCancelUnsavedChanges={handleCancelUnsavedChanges}
    />
  );
}
