
import { useQuery } from '@tanstack/react-query';
import { supabase } from '@/integrations/supabase/client';
import { useAuth } from '@/hooks/useAuth';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

export function useScriptsData() {
  const { user } = useAuth();

  const { data: scripts = [] } = useQuery({
    queryKey: ['scripts'],
    queryFn: async () => {
      if (!user?.id) return [];
      
      const { data, error } = await supabase
        .from('scripts')
        .select('*')
        .eq('user_id', user.id)
        .order('created_at', { ascending: false });

      if (error) {
        console.error('Error fetching scripts:', error);
        throw error;
      }

      return data.map(script => ({
        id: script.id,
        name: script.name || 'Untitled',
        referenceDate: new Date(script.reference_date),
        status: script.status,
        events: []
      }));
    },
    enabled: !!user?.id
  });

  return { scripts };
}
