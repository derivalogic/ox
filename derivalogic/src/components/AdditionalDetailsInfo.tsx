
import React from 'react';
import { FormField, FormItem, FormLabel, FormControl, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
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

interface AdditionalDetailsInfoProps {
  control: Control<TransactionFormData>;
}

export function AdditionalDetailsInfo({ control }: AdditionalDetailsInfoProps) {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-900 border-b border-gray-200 pb-2">
        Additional Details
      </h2>
      <div className="grid gap-4">
        <FormField
          control={control}
          name="notes"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Notes</FormLabel>
              <FormControl>
                <Textarea {...field} placeholder="Enter notes" rows={4} />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name="lastModified"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Last Modified</FormLabel>
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
