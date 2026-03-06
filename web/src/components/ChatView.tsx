"use client";

import { motion } from "framer-motion";
import { Send, User, Wand2 } from "lucide-react";
import { useState } from "react";

export interface Message {
  id: string;
  role: "user" | "assistant";
  content: string;
}

export function ChatView({ messages, onSendMessage }: { messages: Message[], onSendMessage: (msg: string) => void }) {
  const [input, setInput] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim()) return;
    onSendMessage(input);
    setInput("");
  };

  return (
    <div className="flex flex-col h-full bg-zinc-950 text-zinc-100 font-sans">
      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {messages.map((msg) => (
          <motion.div
            key={msg.id}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className={`flex gap-4 ${msg.role === "user" ? "justify-end" : "justify-start"}`}
          >
            {msg.role === "assistant" && (
              <div className="w-8 h-8 rounded bg-cyan-500/10 flex items-center justify-center border border-cyan-500/20">
                <Wand2 className="w-4 h-4 text-cyan-400" />
              </div>
            )}
            <div className={`max-w-[80%] rounded-lg p-4 text-sm leading-relaxed ${
              msg.role === "user" 
                ? "bg-zinc-800 border border-white/5 text-zinc-200" 
                : "bg-black/40 border border-white/5 text-zinc-300"
            }`}>
              {msg.content}
            </div>
            {msg.role === "user" && (
              <div className="w-8 h-8 rounded bg-yellow-500/10 flex items-center justify-center border border-yellow-500/20">
                <User className="w-4 h-4 text-yellow-500" />
              </div>
            )}
          </motion.div>
        ))}
      </div>

      <div className="p-4 border-t border-white/5 bg-zinc-900/50">
        <form onSubmit={handleSubmit} className="relative max-w-4xl mx-auto">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Type your instructions, Guv'nor..."
            className="w-full bg-black/50 border border-white/10 rounded-xl py-4 pl-6 pr-12 text-sm focus:outline-none focus:border-cyan-500/50 transition-colors placeholder:text-zinc-600"
          />
          <button
            type="submit"
            className="absolute right-3 top-1/2 -translate-y-1/2 p-2 hover:bg-white/5 rounded-lg transition-colors group"
          >
            <Send className="w-4 h-4 text-zinc-500 group-hover:text-cyan-400" />
          </button>
        </form>
      </div>
    </div>
  );
}
