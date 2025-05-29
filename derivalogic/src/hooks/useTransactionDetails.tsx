
import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { useQuery } from '@tanstack/react-query';
import { useForm } from 'react-hook-form';
import { supabase } from '@/integrations/supabase/client';
import { useAuth } from '@/hooks/useAuth';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

interface TransactionFormData {
  scriptName: string;
  scriptId: string;
  status: string;
  createdDate: string;
  transactionType: string;
  referenceDate: string;
  eventsCount: string;
  counterpartyName: string;
  contactInformation: string;
  additionalDetails: string;
  notes: string;
  lastModified: string;
}

const formatDateYYYYMMDD = (date: Date) => {
  return date.toISOString().split('T')[0];
};

export function useTransactionDetails() {
  const location = useLocation();
  const navigate = useNavigate();
  const script = location.state?.script as Script;
  const { user } = useAuth();

  const form = useForm<TransactionFormData>({
    defaultValues: {
      scriptName: '',
      scriptId: '',
      status: '',
      createdDate: '',
      transactionType: 'Financial Derivative',
      referenceDate: '',
      eventsCount: '',
      counterpartyName: '',
      contactInformation: '',
      additionalDetails: '',
      notes: '',
      lastModified: ''
    }
  });

  useEffect(() => {
    if (!script || !user) {
      navigate('/');
    } else {
      // Populate form with script data
      form.setValue('scriptName', script.name || '');
      form.setValue('scriptId', script.id);
      form.setValue('status', script.status || 'DRAFT');
      form.setValue('createdDate', formatDateYYYYMMDD(script.referenceDate));
      form.setValue('referenceDate', formatDateYYYYMMDD(script.referenceDate));
      form.setValue('eventsCount', (script.events?.length || 0).toString());
      form.setValue('lastModified', formatDateYYYYMMDD(script.referenceDate));
    }
  }, [script, user, navigate, form]);

  // Fetch all scripts for the sidebar
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

  const handleScriptSelect = (selectedScript: Script) => {
    navigate('/transaction-details', { state: { script: selectedScript } });
  };

  const handleNewScript = () => {
    navigate('/');
  };

  const onSubmit = (data: TransactionFormData) => {
    console.log('Form submitted:', data);
    // Handle form submission here
  };

  return {
    script,
    user,
    scripts,
    form,
    handleScriptSelect,
    handleNewScript,
    onSubmit
  };
}
