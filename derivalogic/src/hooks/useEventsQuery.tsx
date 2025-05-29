
import { useQuery } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
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

export function useEventsQuery(script: Script | null) {
  const { user } = useAuth();

  const { data: events = [], isLoading } = useQuery({
    queryKey: ['events', script?.id],
    queryFn: async () => {
      if (!script?.id || !user?.id) return [];
      
      const { data, error } = await supabase
        .from('events')
        .select('*')
        .eq('script_id', script.id)
        .eq('user_id', user.id)
        .order('event_date', { ascending: true });

      if (error) {
        console.error('Error fetching events:', error);
        throw error;
      }

      return data.map(event => ({
        id: event.id,
        eventDate: new Date(event.event_date),
        script: event.script_content,
        name: event.name,
        description: event.description
      }));
    },
    enabled: !!script?.id && !!user?.id
  });

  return { events, isLoading };
}
