"use client";

import { useState, useEffect } from "react";
import { ChatView, type Message } from "@/components/ChatView";
import { AgentActivity, type ActivityItem } from "@/components/AgentActivity";
import { Shield, Zap, Database, Terminal as TerminalIcon, History, Settings, Cpu } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

export default function Home() {
  const [messages, setMessages] = useState<Message[]>([
    { id: "1", role: "assistant", content: "✦ Guv'nor: Standing by. How can I help you build today?" }
  ]);

  const [activities, setActivities] = useState<ActivityItem[]>([
    { id: "1", agent: "System", action: "Awaiting instructions...", status: "pending", timestamp: "now" }
  ]);

  const [isSidebarOpen, setIsSidebarOpen] = useState(true);

  const handleSendMessage = (content: string) => {
    const userMsg: Message = { id: Date.now().toString(), role: "user", content };
    setMessages(prev => [...prev, userMsg]);

    // Simulate Codebuff-style Agent Workflow
    const planId = Date.now().toString();
    const newActivity: ActivityItem = {
      id: planId,
      agent: "Planner (Gemini)",
      action: `Analyzing repository for: "${content}"`,
      status: "running",
      timestamp: "now"
    };
    setActivities(prev => [newActivity, ...prev]);

    setTimeout(() => {
      setActivities(prev => prev.map(a => a.id === planId ? { ...a, status: "completed", action: "✔ Planner: Identified 3 target files." } : a));
      
      const coderId = (Date.now() + 1).toString();
      const coderActivity: ActivityItem = {
        id: coderId,
        agent: "Coder (Opus)",
        action: "Generating surgical SEARCH/REPLACE blocks...",
        status: "running",
        timestamp: "now"
      };
      setActivities(prev => [coderActivity, ...prev]);

      // Simulate streaming response
      const assistantMsgId = (Date.now() + 2).toString();
      setMessages(prev => [...prev, { id: assistantMsgId, role: "assistant", content: "✦ Guv: Analyzing changes... " }]);
      
      let tokens = ["I've ", "prepared ", "the ", "necessary ", "patches ", "for ", "your ", "review. ", "The ", "build ", "looks ", "clean."];
      let i = 0;
      const interval = setInterval(() => {
        if (i < tokens.length) {
          setMessages(prev => prev.map(m => m.id === assistantMsgId ? { ...m, content: m.content + tokens[i] } : m));
          i++;
        } else {
          clearInterval(interval);
          setActivities(prev => prev.map(a => a.id === coderId ? { ...a, status: "completed", action: "✔ Coder: Patch generated for src/main.rs" } : a));
          
          const reviewerId = (Date.now() + 3).toString();
          setActivities(prev => [{
            id: reviewerId,
            agent: "Reviewer (Local)",
            action: "✔ Reviewer: Validated build and syntax.",
            status: "completed",
            timestamp: "now"
          }, ...prev]);
        }
      }, 100);
    }, 1200);
  };

  return (
    <main className="flex h-screen w-screen bg-zinc-950 overflow-hidden text-zinc-100 font-sans selection:bg-cyan-500/30">
      {/* Sidebar Navigation - Crush Style */}
      <div className="w-16 flex flex-col items-center py-6 gap-8 border-r border-white/[0.03] bg-zinc-900/40 backdrop-blur-xl z-20">
        <motion.div 
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          className="w-10 h-10 rounded-xl bg-cyan-500/20 flex items-center justify-center border border-cyan-500/30 shadow-[0_0_20px_rgba(6,182,212,0.15)]"
        >
          <span className="text-xl">🎩</span>
        </motion.div>
        
        <nav className="flex flex-col gap-6 flex-1">
          <NavItem icon={<TerminalIcon className="w-5 h-5" />} active />
          <NavItem icon={<History className="w-5 h-5" />} />
          <NavItem icon={<Database className="w-5 h-5" />} />
          <NavItem icon={<Cpu className="w-5 h-5" />} />
        </nav>

        <div className="flex flex-col gap-6 mt-auto">
          <NavItem icon={<Settings className="w-5 h-5" />} />
          <div className="w-8 h-8 rounded-full bg-gradient-to-tr from-cyan-500 to-blue-500 border border-white/10" />
        </div>
      </div>

      {/* History Sidebar - Codebuff style toggle */}
      <AnimatePresence>
        {isSidebarOpen && (
          <motion.div 
            initial={{ width: 0, opacity: 0 }}
            animate={{ width: 260, opacity: 1 }}
            exit={{ width: 0, opacity: 0 }}
            className="flex flex-col border-r border-white/[0.03] bg-zinc-900/20 backdrop-blur-md overflow-hidden"
          >
            <div className="p-6">
              <h2 className="text-xs font-bold uppercase tracking-widest text-zinc-500 mb-6">Recent Sessions</h2>
              <div className="space-y-2">
                <SessionItem title="Add dark mode toggle" date="2m ago" active />
                <SessionItem title="Refactor auth middleware" date="1h ago" />
                <SessionItem title="Fix tree-sitter bindings" date="3h ago" />
                <SessionItem title="Update globals.css" date="Yesterday" />
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col relative">
        <header className="h-14 border-b border-white/[0.03] flex items-center px-8 justify-between bg-zinc-900/10 backdrop-blur-sm z-10">
          <div className="flex items-center gap-4">
            <button 
              onClick={() => setIsSidebarOpen(!isSidebarOpen)}
              className="p-1.5 hover:bg-white/5 rounded-md transition-colors text-zinc-500"
            >
              <History className="w-4 h-4" />
            </button>
            <div className="flex items-center gap-3">
              <h1 className="font-semibold tracking-tight text-sm text-zinc-300">GUV-Code <span className="text-zinc-600 font-normal ml-2">/ guv-code</span></h1>
              <span className="px-1.5 py-0.5 rounded-full text-[9px] bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 font-bold uppercase tracking-tighter">v0.2.0</span>
            </div>
          </div>
          
          <div className="flex items-center gap-6">
            <div className="flex items-center gap-4 text-[11px] font-medium">
              <div className="flex items-center gap-2 text-zinc-400">
                <div className="w-1.5 h-1.5 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.5)]" />
                Gemini 3.1 Pro
              </div>
              <div className="flex items-center gap-2 text-zinc-400">
                <div className="w-1.5 h-1.5 rounded-full bg-cyan-500 shadow-[0_0_8px_rgba(6,182,212,0.5)]" />
                Claude 3.7 Opus
              </div>
            </div>
            <div className="w-px h-4 bg-white/10" />
            <div className="text-[11px] font-mono text-zinc-500 tabular-nums">
              Budget: <span className="text-emerald-500/80">$4.50</span> <span className="text-zinc-700">/ $10.00</span>
            </div>
          </div>
        </header>
        
        <div className="flex-1 overflow-hidden flex">
          <div className="flex-1 overflow-hidden relative">
            <ChatView messages={messages} onSendMessage={handleSendMessage} />
          </div>
          <div className="w-72 h-full bg-black/10 backdrop-blur-sm">
            <AgentActivity activities={activities} />
          </div>
        </div>
      </div>
    </main>
  );
}

function NavItem({ icon, active }: { icon: React.ReactNode, active?: boolean }) {
  return (
    <motion.div 
      whileHover={{ scale: 1.1 }}
      className={`p-2.5 rounded-xl cursor-pointer transition-all duration-200 ${
        active 
          ? "text-cyan-400 bg-cyan-500/10 border border-cyan-500/20 shadow-[0_0_15px_rgba(6,182,212,0.1)]" 
          : "text-zinc-600 hover:text-zinc-400 hover:bg-white/5"
      }`}
    >
      {icon}
    </motion.div>
  );
}

function SessionItem({ title, date, active }: { title: string, date: string, active?: boolean }) {
  return (
    <div className={`p-3 rounded-lg cursor-pointer group transition-all duration-200 border ${
      active 
        ? "bg-cyan-500/5 border-cyan-500/10" 
        : "border-transparent hover:bg-white/[0.02]"
    }`}>
      <div className={`text-xs font-medium truncate mb-1 ${active ? "text-cyan-400" : "text-zinc-400 group-hover:text-zinc-300"}`}>
        {title}
      </div>
      <div className="text-[10px] text-zinc-600 tabular-nums">{date}</div>
    </div>
  );
}
