
import React from 'react';
import { format } from "date-fns";
import { CalendarIcon } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

interface DateInputProps {
  helperText?: string;
  error?: boolean;
  onDateChange: (value: Date | null) => void;
  value?: Date | null;
  label?: string;
  disabled?: boolean;
}

export default function DateInput({ 
  helperText, 
  error, 
  onDateChange, 
  value, 
  label = "Event Date",
  disabled = false
}: DateInputProps) {
  return (
    <div className="flex flex-col space-y-2">
      <Popover>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            disabled={disabled}
            className={cn(
              "w-full justify-start text-left font-normal bg-white border-gray-300 text-gray-900 hover:bg-gray-50",
              !value && "text-gray-400",
              error && "border-red-500",
              disabled && "opacity-50 cursor-not-allowed"
            )}
          >
            <CalendarIcon className="mr-2 h-4 w-4" />
            {value ? format(value, "yyyy/MM/dd") : <span>Pick a date</span>}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0 bg-white border-gray-200" align="start">
          <Calendar
            mode="single"
            selected={value || undefined}
            onSelect={(date) => onDateChange(date || null)}
            initialFocus
            className="pointer-events-auto text-gray-900"
          />
        </PopoverContent>
      </Popover>
      {error && helperText && (
        <p className="text-sm text-red-500">{helperText}</p>
      )}
    </div>
  );
}
