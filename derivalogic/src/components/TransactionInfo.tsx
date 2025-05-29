
import React from 'react';
import { FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Control } from 'react-hook-form';

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

interface TransactionInfoProps {
  control: Control<TransactionFormData>;
}

export function TransactionInfo({ control }: TransactionInfoProps) {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-900 border-b border-gray-200 pb-2">
        Transaction Information
      </h2>
      <div className="grid gap-4 md:grid-cols-2">
        <FormField
          control={control}
          name="transactionType"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Transaction Type</FormLabel>
              <FormControl>
                <Input {...field} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name="referenceDate"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Reference Date</FormLabel>
              <FormControl>
                <Input {...field} readOnly className="bg-gray-50" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name="eventsCount"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Events Count</FormLabel>
              <FormControl>
                <Input {...field} readOnly className="bg-gray-50" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
      </div>
    </div>
  );
}
