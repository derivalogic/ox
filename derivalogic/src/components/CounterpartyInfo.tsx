
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

interface CounterpartyInfoProps {
  control: Control<TransactionFormData>;
}

export function CounterpartyInfo({ control }: CounterpartyInfoProps) {
  return (
    <div className="space-y-6">
      <h2 className="text-lg font-semibold text-gray-900 border-b border-gray-200 pb-2">
        Counterparty Information
      </h2>
      <div className="grid gap-4 md:grid-cols-2">
        <FormField
          control={control}
          name="counterpartyName"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Counterparty Name</FormLabel>
              <FormControl>
                <Input {...field} placeholder="Enter counterparty name" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={control}
          name="contactInformation"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Contact Information</FormLabel>
              <FormControl>
                <Input {...field} placeholder="Enter contact information" />
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />
      </div>
      <FormField
        control={control}
        name="additionalDetails"
        render={({ field }) => (
          <FormItem>
            <FormLabel>Additional Details</FormLabel>
            <FormControl>
              <Textarea {...field} placeholder="Enter additional details" rows={3} />
            </FormControl>
            <FormMessage />
          </FormItem>
        )}
      />
    </div>
  );
}
