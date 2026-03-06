"use client";

import { motion } from "framer-motion";
import { CheckCircle2, Circle, Loader2, XCircle } from "lucide-react";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type ActivityStatus = "pending" | "running" | "completed" | "error";

export interface ActivityItem {
  id: string;
  agent: string;
  action: string;
  status: ActivityStatus;
  timestamp: string;
}

export function AgentActivity({ activities }: { activities: ActivityItem[] }) {
  return (
    <div className="flex flex-col gap-4 p-4 border-l border-white/10 bg-black/20 h-full overflow-y-auto">
      <h3 className="text-xs font-bold uppercase tracking-widest text-zinc-500 mb-2">Agent Activity</h3>
      <div className="space-y-4">
        {activities.map((item, index) => (
          <motion.div
            key={item.id}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: index * 0.1 }}
            className="flex gap-3 items-start group"
          >
            <div className="mt-1">
              {item.status === "completed" && <CheckCircle2 className="w-4 h-4 text-emerald-500" />}
              {item.status === "running" && <Loader2 className="w-4 h-4 text-cyan-500 animate-spin" />}
              {item.status === "pending" && <Circle className="w-4 h-4 text-zinc-600" />}
              {item.status === "error" && <XCircle className="w-4 h-4 text-rose-500" />}
            </div>
            <div className="flex flex-col">
              <span className={cn(
                "text-xs font-semibold",
                item.status === "running" ? "text-cyan-400" : "text-zinc-300"
              )}>
                {item.agent}
              </span>
              <span className="text-xs text-zinc-500 leading-relaxed">
                {item.action}
              </span>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
