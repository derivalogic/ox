
import React from 'react';
import { ScriptingEvent } from '@/components/ScriptingEvent';

interface Event {
  id: string;
  eventDate: Date;
  script: string;
  name?: string;
  description?: string;
}

interface EventsTimelineProps {
  events: Event[];
  isModifyMode: boolean;
  onScriptChange: (eventId: string, value: string) => void;
  onDateChange: (eventId: string, value: Date | null) => void;
  onNameChange: (eventId: string, name: string) => void;
  onDescriptionChange: (eventId: string, description: string) => void;
  onDeleteEvent: (eventId: string) => void;
}

export function EventsTimeline({
  events,
  isModifyMode,
  onScriptChange,
  onDateChange,
  onNameChange,
  onDescriptionChange,
  onDeleteEvent
}: EventsTimelineProps) {
  if (events.length === 0) {
    return (
      <div className="text-center py-16">
        <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <div className="w-8 h-8 border-2 border-gray-300 border-dashed rounded-full"></div>
        </div>
        <h3 className="text-lg font-semibold text-gray-700 mb-2">No events yet</h3>
        <p className="text-gray-500 max-w-md mx-auto leading-relaxed">
          Click "Add Event" to create your first transaction event and start building your script timeline.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6 relative pb-12">
      {/* Timeline line with gradient */}
      <div className="absolute left-8 top-0 bottom-12 w-1 bg-gradient-to-b from-blue-500 via-blue-400 to-blue-300 rounded-full shadow-sm"></div>
      
      {events.map((event, index) => (
        <div key={event.id} className="relative">
          {/* Timeline dot with enhanced styling */}
          <div className="absolute left-6 top-12 w-5 h-5 bg-gradient-to-br from-blue-500 to-blue-600 rounded-full border-3 border-white shadow-lg z-10 ring-4 ring-blue-100/60"></div>
          
          <div className="ml-20 transform transition-all duration-300 hover:translate-x-1">
            <ScriptingEvent
              id={event.id}
              initialScript={event.script}
              initialDate={event.eventDate}
              initialName={event.name}
              initialDescription={event.description}
              eventNumber={index + 1}
              onScriptChange={(value) => onScriptChange(event.id, value)}
              onDateChange={(value) => onDateChange(event.id, value)}
              onNameChange={(value) => onNameChange(event.id, value)}
              onDescriptionChange={(value) => onDescriptionChange(event.id, value)}
              onDelete={() => onDeleteEvent(event.id)}
              scriptError={null}
              dateError={null}
              readOnly={!isModifyMode}
            />
          </div>
        </div>
      ))}
      
      {/* Timeline endpoint with enhanced styling */}
      <div className="absolute left-6 bottom-0 w-5 h-5 bg-gradient-to-br from-blue-300 to-blue-400 rounded-full border-3 border-white shadow-lg z-10 ring-4 ring-blue-100/40"></div>
    </div>
  );
}
