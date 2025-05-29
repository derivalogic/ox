
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
import { useToast } from '@/hooks/use-toast';
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

export function useEventMutations(script: Script | null) {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { user } = useAuth();

  // Create event mutation
  const createEventMutation = useMutation({
    mutationFn: async (newEvent: Omit<Event, 'id'>) => {
      if (!user?.id) throw new Error('User not authenticated');
      
      const { data, error } = await supabase
        .from('events')
        .insert({
          script_id: script.id,
          user_id: user.id,
          name: newEvent.name,
          description: newEvent.description,
          event_date: newEvent.eventDate.toISOString(),
          script_content: newEvent.script
        })
        .select()
        .single();

      if (error) throw error;
      return data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['events', script?.id] });
      toast({ title: "Event created successfully" });
    },
    onError: (error) => {
      console.error('Error creating event:', error);
      toast({ title: "Error creating event", variant: "destructive" });
    }
  });

  // Update event mutation
  const updateEventMutation = useMutation({
    mutationFn: async ({ eventId, updates }: { eventId: string; updates: Partial<Event> }) => {
      if (!user?.id) throw new Error('User not authenticated');
      
      const { error } = await supabase
        .from('events')
        .update({
          name: updates.name,
          description: updates.description,
          event_date: updates.eventDate?.toISOString(),
          script_content: updates.script
        })
        .eq('id', eventId)
        .eq('user_id', user.id);

      if (error) throw error;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['events', script?.id] });
      toast({ title: "Event updated successfully" });
    },
    onError: (error) => {
      console.error('Error updating event:', error);
      toast({ title: "Error updating event", variant: "destructive" });
    }
  });

  // Delete event mutation
  const deleteEventMutation = useMutation({
    mutationFn: async (eventId: string) => {
      if (!user?.id) throw new Error('User not authenticated');
      
      const { error } = await supabase
        .from('events')
        .delete()
        .eq('id', eventId)
        .eq('user_id', user.id);

      if (error) throw error;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['events', script?.id] });
      toast({ title: "Event deleted successfully" });
    },
    onError: (error) => {
      console.error('Error deleting event:', error);
      toast({ title: "Error deleting event", variant: "destructive" });
    }
  });

  return {
    createEventMutation,
    updateEventMutation,
    deleteEventMutation
  };
}
