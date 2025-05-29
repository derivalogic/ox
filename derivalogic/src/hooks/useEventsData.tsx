
import { useScriptsData } from '@/hooks/useScriptsData';
import { useEventsQuery } from '@/hooks/useEventsQuery';
import { useScriptMutations } from '@/hooks/useScriptMutations';
import { useEventMutations } from '@/hooks/useEventMutations';

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

export function useEventsData(script: Script | null) {
  const { scripts } = useScriptsData();
  const { events, isLoading } = useEventsQuery(script);
  const { saveScriptMutation, createScriptMutation, deleteDraftScriptMutation } = useScriptMutations(script);
  const { createEventMutation, updateEventMutation, deleteEventMutation } = useEventMutations(script);

  return {
    scripts,
    events,
    isLoading,
    saveScriptMutation,
    createScriptMutation,
    createEventMutation,
    updateEventMutation,
    deleteEventMutation,
    deleteDraftScriptMutation
  };
}
