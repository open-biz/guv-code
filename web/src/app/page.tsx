"use client";

import { useState } from "react";
import { ChatView, type Message } from "@/components/ChatView";
import { AgentActivity, type ActivityItem } from "@/components/AgentActivity";
import { Shield, Zap, Database, Terminal as TerminalIcon } from "lucide-react";

export default function Home() {
  const [messages, setMessages] = useState<Message[]>([
    { id: "1", role: "assistant", content: "Right away, Guv'nor. How can I help you today?" }
  ]);

  const [activities, setActivities] = useState<ActivityItem[]>([
    { id: "1", agent: "System", action: "Awaiting instructions...", status: "pending", timestamp: "now" }
  ]);

  const handleSendMessage = (content: string) => {
    const userMsg: Message = { id: Date.now().toString(), role: "user", content };
    setMessages(prev => [...prev, userMsg]);

    // Simulate Agent Workflow
    const newActivity: ActivityItem = {
      id: Date.now().toString(),
      agent: "Planner",
      action: `Analyzing request: "${content}"`,
      status: "running",
      timestamp: "now"
    };
    setActivities(prev => [newActivity, ...prev]);

    setTimeout(() => {
      setActivities(prev => prev.map(a => a.id === newActivity.id ? { ...a, status: "completed" } : a));
      const coderActivity: ActivityItem = {
        id: (Date.now() + 1).toString(),
        agent: "Coder",
        action: "Generating AST diffs for src/main.rs",
        status: "running",
        timestamp: "now"
      };
      setActivities(prev => [coderActivity, ...prev]);

      setTimeout(() => {
        setActivities(prev => prev.map(a => a.id === coderActivity.id ? { ...a, status: "completed" } : a));
        setMessages(prev => [...prev, {
          id: (Date.now() + 2).toString(),
          role: "assistant",
          content: "I've analyzed the request and prepared the necessary patches. You can review them in the terminal or apply them here."
        }]);
      }, 1500);
    }, 1000);
  };

  return (
    <main className="flex h-screen w-screen bg-zinc-950 overflow-hidden text-zinc-100">
      {/* Sidebar Navigation */}
      <div className="w-16 flex flex-col items-center py-6 gap-8 border-r border-white/5 bg-zinc-900/20">
        <div className="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center border border-cyan-500/40">
          <span className="text-xl">🎩</span>
        </div>
        <nav className="flex flex-col gap-6">
          <TerminalIcon className="w-5 h-5 text-cyan-500" />
          <Database className="w-5 h-5 text-zinc-600 hover:text-zinc-400 cursor-pointer" />
          <Zap className="w-5 h-5 text-zinc-600 hover:text-zinc-400 cursor-pointer" />
          <Shield className="w-5 h-5 text-zinc-600 hover:text-zinc-400 cursor-pointer" />
        </nav>
      </div>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col">
        <header className="h-16 border-b border-white/5 flex items-center px-8 justify-between bg-zinc-900/10">
          <div className="flex items-center gap-3">
            <h1 className="font-bold tracking-tight text-sm">GUV-Code</h1>
            <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-500/10 text-emerald-500 border border-emerald-500/20 font-bold uppercase">Active</span>
          </div>
          <div className="flex items-center gap-4 text-xs text-zinc-500">
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-cyan-500 animate-pulse" />
              Gemini Pro
            </div>
            <div className="w-px h-4 bg-white/10" />
            <div className="font-mono">$4.50 / $10.00</div>
          </div>
        </header>
        
        <div className="flex-1 overflow-hidden flex">
          <div className="flex-1 overflow-hidden">
            <ChatView messages={messages} onSendMessage={handleSendMessage} />
          </div>
          <div className="w-80 h-full">
            <AgentActivity activities={activities} />
          </div>
        </div>
      </div>
    </main>
  );
}
