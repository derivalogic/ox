import { useCallback } from 'react';

interface Event {
  id: string;
  eventDate: Date;
  script: string;
  name?: string;
  description?: string;
}

interface EventsHandlersProps {
  isModifyMode: boolean;
  createEventMutation: any;
  updateEventMutation: any;
  deleteEventMutation: any;
  saveScriptMutation: any;
  setIsSaveModalOpen: (open: boolean) => void;
  setIsModifyMode: () => void;
  setShowSavedMessage: (show: boolean) => void;
  setCurrentScriptName: (name: string) => void;
  script?: any;
}

export function useEventsHandlers({
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
}: EventsHandlersProps) {
  const handleAddEvent = useCallback(() => {
    if (!isModifyMode) return;
    
    const newEvent: Omit<Event, 'id'> = {
      eventDate: new Date(),
      script: '',
      name: 'Event',
      description: ''
    };
    createEventMutation.mutate(newEvent);
  }, [isModifyMode, createEventMutation]);

  const handleDeleteEvent = useCallback((eventId: string) => {
    if (!isModifyMode) return;
    deleteEventMutation.mutate(eventId);
  }, [isModifyMode, deleteEventMutation]);

  const handleScriptChange = useCallback((eventId: string, value: string) => {
    if (!isModifyMode) return;
    updateEventMutation.mutate({
      eventId,
      updates: { script: value }
    });
  }, [isModifyMode, updateEventMutation]);

  const handleDateChange = useCallback((eventId: string, value: Date | null) => {
    if (!isModifyMode || !value) return;
    updateEventMutation.mutate({
      eventId,
      updates: { eventDate: value }
    });
  }, [isModifyMode, updateEventMutation]);

  const handleNameChange = useCallback((eventId: string, name: string) => {
    if (!isModifyMode) return;
    updateEventMutation.mutate({
      eventId,
      updates: { name }
    });
  }, [isModifyMode, updateEventMutation]);

  const handleDescriptionChange = useCallback((eventId: string, description: string) => {
    if (!isModifyMode) return;
    updateEventMutation.mutate({
      eventId,
      updates: { description }
    });
  }, [isModifyMode, updateEventMutation]);

  const handleScriptNameChange = useCallback((name: string) => {
    console.log('Script name changed to:', name);
    setCurrentScriptName(name);
  }, [setCurrentScriptName]);

  const handleSave = useCallback((currentScriptName?: string) => {
    const nameToSave = currentScriptName || script?.name || 'Untitled';
    console.log('Saving script with name:', nameToSave);
    
    saveScriptMutation.mutate({ 
      scriptName: nameToSave, 
      transactionDetails: '' 
    });
  }, [script?.name, saveScriptMutation]);

  const handleModify = useCallback(() => {
    console.log('Entering modify mode for saved script');
    setIsModifyMode();
    setShowSavedMessage(false);
  }, [setIsModifyMode, setShowSavedMessage]);

  const handleSaveScript = useCallback((scriptName: string, transactionDetails: string) => {
    console.log('Saving script with name from modal:', scriptName);
    saveScriptMutation.mutate({ scriptName, transactionDetails });
  }, [saveScriptMutation]);

  const handleRun = useCallback(() => {
    console.log('Running evaluation for script:', script?.id, 'with events:', script?.events);
  }, [script]);

  return {
    handleAddEvent,
    handleDeleteEvent,
    handleScriptChange,
    handleDateChange,
    handleNameChange,
    handleDescriptionChange,
    handleScriptNameChange,
    handleSave,
    handleModify,
    handleSaveScript,
    handleRun
  };
}
