
import React from 'react';
import { Button } from "@/components/ui/button";
import { Plus, Calendar, Play } from "lucide-react";
import { EventsTimeline } from './EventsTimeline';

interface Event {
  id: string;
  eventDate: Date;
  script: string;
  name?: string;
  description?: string;
}

interface EventsContentProps {
  events: Event[];
  isModifyMode: boolean;
  isCreatingEvent: boolean;
  onAddEvent: () => void;
  onRun: () => void;
  onScriptChange: (eventId: string, value: string) => void;
  onDateChange: (eventId: string, value: Date | null) => void;
  onNameChange: (eventId: string, name: string) => void;
  onDescriptionChange: (eventId: string, description: string) => void;
  onDeleteEvent: (eventId: string) => void;
}

export function EventsContent({
  events,
  isModifyMode,
  isCreatingEvent,
  onAddEvent,
  onRun,
  onScriptChange,
  onDateChange,
  onNameChange,
  onDescriptionChange,
  onDeleteEvent
}: EventsContentProps) {
  return (
    <>
      {/* Transaction Events Header with Action Buttons */}
      <div className="flex items-center justify-between mb-8 p-6 bg-gradient-to-r from-gray-50/80 to-white rounded-xl border border-gray-200/60">
        <div className="flex items-center gap-4">
          <div className="p-2 bg-primary/10 rounded-xl">
            <Calendar className="h-6 w-6 text-primary" />
          </div>
          <div>
            <h2 className="text-2xl font-bold text-gray-900 tracking-tight">Transaction Events</h2>
            <p className="text-sm text-gray-600 mt-1">
              {events.length} {events.length === 1 ? 'event' : 'events'} configured
            </p>
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Button
            onClick={onAddEvent}
            disabled={isCreatingEvent || !isModifyMode}
            variant="outline"
            className="border-2 border-primary/20 text-primary hover:bg-primary/10 hover:border-primary/30 flex items-center gap-2 h-11 px-5 rounded-xl transition-all duration-200 font-medium shadow-sm hover:shadow-md disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Plus className="h-4 w-4" />
            Add Event
          </Button>
          <Button
            onClick={onRun}
            className="bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 text-white h-11 px-6 rounded-xl shadow-md hover:shadow-lg transition-all duration-200 font-medium flex items-center gap-2"
          >
            <Play className="h-4 w-4" />
            RUN
          </Button>
        </div>
      </div>

      {/* Events Timeline */}
      <EventsTimeline
        events={events}
        isModifyMode={isModifyMode}
        onScriptChange={onScriptChange}
        onDateChange={onDateChange}
        onNameChange={onNameChange}
        onDescriptionChange={onDescriptionChange}
        onDeleteEvent={onDeleteEvent}
      />
    </>
  );
}
