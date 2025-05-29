
import React, { useState } from 'react';
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { ChevronDown, ChevronUp, Calendar, Trash2, Edit3 } from "lucide-react";
import { format } from "date-fns";
import DateInput from './DateInput';
import ScriptingArea from './ScriptingArea';
import { EventEditModal } from './EventEditModal';

export interface ScriptingEventProps {
  id: string;
  initialScript: string | null;
  initialDate: Date | null;
  initialName?: string | null;
  initialDescription?: string | null;
  eventNumber: number;
  onScriptChange: (value: string) => void;
  onDateChange: (value: Date | null) => void;
  onNameChange?: (value: string) => void;
  onDescriptionChange?: (value: string) => void;
  onDelete: () => void;
  scriptError: string | null;
  dateError: string | null;
  readOnly?: boolean;
}

export function ScriptingEvent({
  id,
  initialScript,
  initialDate,
  initialName,
  initialDescription,
  eventNumber,
  onScriptChange,
  onDateChange,
  onNameChange,
  onDescriptionChange,
  onDelete,
  scriptError,
  dateError,
  readOnly = false
}: ScriptingEventProps) {
  const [isOpen, setIsOpen] = useState(true);
  const [script, setScript] = useState<string>(initialScript || '');
  const [date, setDate] = useState<Date | null>(initialDate);
  const [name, setName] = useState<string>(initialName || `Event ${eventNumber}`);
  const [description, setDescription] = useState<string>(initialDescription || '');
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);

  const handleScriptChange = (value: string) => {
    if (readOnly) return;
    setScript(value);
    onScriptChange(value);
  };

  const handleDateChange = (value: Date | null) => {
    if (readOnly) return;
    setDate(value);
    onDateChange(value);
  };

  const handleEditSave = (newName: string, newDescription: string) => {
    if (readOnly) return;
    setName(newName);
    setDescription(newDescription);
    if (onNameChange) onNameChange(newName);
    if (onDescriptionChange) onDescriptionChange(newDescription);
  };

  return (
    <>
      <Card className="mb-6 bg-white border border-gray-200 shadow-sm hover:shadow-md transition-all duration-200 rounded-lg overflow-hidden">
        <Collapsible open={isOpen} onOpenChange={setIsOpen}>
          <CollapsibleTrigger asChild>
            <CardHeader className="flex flex-row items-center justify-between p-6 cursor-pointer hover:bg-gray-50 transition-colors border-b border-gray-100">
              <div className="flex items-center gap-4">
                <div className="w-8 h-8 bg-blue-500 rounded-lg flex items-center justify-center text-sm font-bold text-white">
                  {eventNumber}
                </div>
                <div className="flex flex-col items-start gap-1">
                  <div className="flex items-center gap-3">
                    <h3 className="font-semibold text-gray-900 text-lg">{name}</h3>
                    <span className="text-xs text-gray-400 font-mono bg-gray-100 px-2 py-1 rounded">
                      ID: {id.slice(0, 8)}
                    </span>
                  </div>
                  {description && (
                    <p className="text-sm text-gray-600 max-w-md">{description}</p>
                  )}
                  {date && (
                    <Badge variant="outline" className="mt-2 text-xs border-blue-200 text-blue-700 bg-blue-50">
                      <Calendar className="h-3 w-3 mr-1.5" />
                      {format(date, "yyyy-MM-dd")}
                    </Badge>
                  )}
                </div>
              </div>
              <div className="flex items-center gap-2">
                {!readOnly && (
                  <>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        setIsEditModalOpen(true);
                      }}
                      className="text-blue-600 hover:text-blue-700 hover:bg-blue-50 h-9 w-9 p-0 rounded-lg"
                    >
                      <Edit3 className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        onDelete();
                      }}
                      className="text-red-500 hover:text-red-600 hover:bg-red-50 h-9 w-9 p-0 rounded-lg"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </>
                )}
                <div className="ml-2">
                  {isOpen ? (
                    <ChevronUp className="h-5 w-5 text-gray-400" />
                  ) : (
                    <ChevronDown className="h-5 w-5 text-gray-400" />
                  )}
                </div>
              </div>
            </CardHeader>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <CardContent className="p-6 space-y-6 bg-gray-50">
              {/* Date Selection */}
              <div className="space-y-2">
                <label className="text-sm font-medium text-gray-700">Event Date</label>
                <DateInput 
                  onDateChange={handleDateChange} 
                  value={date}
                  error={!!dateError}
                  helperText={dateError || undefined}
                  disabled={readOnly}
                />
              </div>

              {/* Event Logic Section */}
              <div className="space-y-3">
                <h4 className="text-sm font-semibold text-gray-800">Event Logic</h4>
                <div className="h-32 rounded-lg overflow-hidden border border-gray-200">
                  <ScriptingArea 
                    onScriptChange={handleScriptChange}
                    value={script}
                    readOnly={readOnly}
                  />
                </div>
              </div>

              {/* Error Messages */}
              {(dateError || scriptError) && (
                <div className="space-y-2">
                  {dateError && (
                    <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg">
                      <div className="w-2 h-2 bg-red-500 rounded-full"></div>
                      <p className="text-sm text-red-700">{dateError}</p>
                    </div>
                  )}
                  {scriptError && (
                    <div className="flex items-center gap-2 p-3 bg-red-50 border border-red-200 rounded-lg">
                      <div className="w-2 h-2 bg-red-500 rounded-full"></div>
                      <p className="text-sm text-red-700">{scriptError}</p>
                    </div>
                  )}
                </div>
              )}
            </CardContent>
          </CollapsibleContent>
        </Collapsible>
      </Card>

      {!readOnly && (
        <EventEditModal
          open={isEditModalOpen}
          onClose={() => setIsEditModalOpen(false)}
          initialName={name}
          initialDescription={description}
          onSave={handleEditSave}
        />
      )}
    </>
  );
}
