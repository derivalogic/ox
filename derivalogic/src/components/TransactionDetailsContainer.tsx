
import React from 'react';
import { FileText, Save } from 'lucide-react';
import { Form } from '@/components/ui/form';
import { Button } from '@/components/ui/button';
import { TransactionScriptInfo } from './TransactionScriptInfo';
import { TransactionInfo } from './TransactionInfo';
import { CounterpartyInfo } from './CounterpartyInfo';
import { AdditionalDetailsInfo } from './AdditionalDetailsInfo';

interface Script {
  id: string;
  name: string;
  referenceDate: Date;
  status?: string;
  events: any[];
}

interface TransactionDetailsContainerProps {
  script: Script;
  scripts: Script[];
  form: any;
  onSubmit: (data: any) => void;
}

export function TransactionDetailsContainer({ 
  script, 
  scripts, 
  form, 
  onSubmit 
}: TransactionDetailsContainerProps) {
  const currentScriptData = scripts.find(s => s.id === script?.id);
  const scriptStatus = currentScriptData?.status || 'DRAFT';

  return (
    <div className="min-h-screen bg-white shadow-lg text-gray-900 p-6">
      <div className="mb-8">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <FileText className="h-6 w-6 text-blue-600" />
            <h1 className="text-2xl font-semibold leading-none tracking-tight">Transaction Details</h1>
          </div>
          <span className={`px-3 py-1.5 rounded-lg text-sm font-medium h-7 flex items-center ${
            scriptStatus === 'SAVED' 
              ? 'bg-green-500/20 text-green-600' 
              : 'bg-yellow-500/20 text-yellow-600'
          }`}>
            {scriptStatus === 'SAVED' ? '✓ BOOKED' : '⚡ DRAFT'}
          </span>
        </div>
        <p className="text-gray-600">Transaction details for {script.name}</p>
      </div>

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
          <TransactionScriptInfo control={form.control} />
          <TransactionInfo control={form.control} />
          <CounterpartyInfo control={form.control} />
          <AdditionalDetailsInfo control={form.control} />

          {/* Save Button */}
          <div className="flex justify-end pt-6 border-t border-gray-200">
            <Button type="submit" className="flex items-center gap-2">
              <Save className="h-4 w-4" />
              Save Transaction Details
            </Button>
          </div>
        </form>
      </Form>
    </div>
  );
}
