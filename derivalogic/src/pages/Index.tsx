
import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import MainLayout from "@/components/MainLayout";
import { supabase } from '@/integrations/supabase/client';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useToast } from '@/hooks/use-toast';
import { useAuth } from '@/hooks/useAuth';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

const formatDateYYYYMMDD = (date: Date) => {
  return date.toISOString().split('T')[0];
};

export default function Index() {
  const [currentScript, setCurrentScript] = useState<Script | undefined>();
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { user } = useAuth();
  const navigate = useNavigate();

  // Redirect to auth if not logged in
  useEffect(() => {
    if (!user) {
      navigate('/auth');
    }
  }, [user, navigate]);

  // Fetch scripts from Supabase
  const { data: scripts = [], isLoading } = useQuery({
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
        name: script.name,
        referenceDate: new Date(script.reference_date),
        status: script.status,
        events: []
      }));
    },
    enabled: !!user?.id
  });

  // Create script mutation
  const createScriptMutation = useMutation({
    mutationFn: async () => {
      if (!user?.id) throw new Error('User not authenticated');
      
      const { data, error } = await supabase
        .from('scripts')
        .insert({
          name: 'New Script*',
          reference_date: new Date().toISOString(),
          status: 'DRAFT',
          user_id: user.id
        })
        .select()
        .single();

      if (error) throw error;
      return data;
    },
    onSuccess: (newScript) => {
      queryClient.invalidateQueries({ queryKey: ['scripts'] });
      const script: Script = {
        id: newScript.id,
        name: newScript.name,
        referenceDate: new Date(newScript.reference_date),
        status: newScript.status,
        events: []
      };
      setCurrentScript(script);
      // Navigate directly to events page
      navigate('/events', { state: { script } });
      toast({ 
        title: "Script created successfully",
        description: `New script is ready for editing`
      });
    },
    onError: (error) => {
      console.error('Error creating script:', error);
      toast({ title: "Error creating script", variant: "destructive" });
    }
  });

  const handleScriptSelect = (script: Script) => {
    setCurrentScript(script);
    navigate('/events', { state: { script } });
  };

  const handleNewScript = () => {
    createScriptMutation.mutate();
  };

  if (isLoading) {
    return (
      <MainLayout>
        <div className="flex flex-col items-center justify-center h-full space-y-4">
          <div>Loading scripts...</div>
        </div>
      </MainLayout>
    );
  }

  return (
    <MainLayout 
      scripts={scripts}
      currentScript={currentScript}
      onScriptSelect={handleScriptSelect}
      onNewScript={handleNewScript}
    >
      <div className="flex flex-col items-center justify-center h-full space-y-8">
        <div className="text-center space-y-4">
          <h1 className="text-6xl font-bold text-primary">DerivaLogic</h1>
          <p className="text-xl text-muted-foreground font-light">
            Financial derivatives redefined.
          </p>
        </div>
        
        {user && scripts.length > 0 && (
          <div className="w-full max-w-4xl space-y-6">
            <h2 className="text-2xl font-semibold text-gray-900 text-center">Recent Scripts</h2>
            <div className="grid gap-4">
              {scripts.slice(0, 5).map((script) => (
                <div 
                  key={script.id}
                  onClick={() => handleScriptSelect(script)}
                  className="bg-white border border-gray-200 rounded-lg p-6 hover:shadow-md transition-shadow cursor-pointer"
                >
                  <div className="flex items-center justify-between">
                    <div className="space-y-2">
                      <div className="flex items-center gap-3">
                        <h3 className="text-lg font-semibold text-gray-900">{script.name}</h3>
                        {script.status === 'DRAFT' && (
                          <span className="bg-yellow-500/20 text-yellow-600 px-2 py-1 rounded text-xs font-medium">
                            DRAFT
                          </span>
                        )}
                        {script.status === 'SAVED' && (
                          <span className="bg-green-500/20 text-green-600 px-2 py-1 rounded text-xs font-medium">
                            BOOKED
                          </span>
                        )}
                      </div>
                      <div className="text-sm text-gray-500 space-y-1">
                        <div>ID: {script.id.slice(0, 8)}...</div>
                        <div>Created: {formatDateYYYYMMDD(script.referenceDate)}</div>
                      </div>
                    </div>
                    <div className="text-right text-sm text-gray-500">
                      <div>Events: 0</div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {!user && (
          <div className="mt-8">
            <button
              onClick={() => navigate('/auth')}
              className="bg-primary text-primary-foreground px-6 py-3 rounded-lg font-medium hover:bg-primary/90 transition-colors"
            >
              Get Started
            </button>
          </div>
        )}
      </div>
    </MainLayout>
  );
}
