## Collaboration

At the start of every session:
1. Check your current phase and task from the project context (ROADMAP.md, active PLAN.md, or recent git log)
2. Run `collab watch --role "<project>: <current phase/task>"` 45 use real context, not a generic description
   Example: `collab watch --role "yubitui: phase 09 OathScreen widget implementation"`
3. Run `collab roster` to see who else is online and what they're doing
4. When your focus changes, restart watch with an updated --role reflecting the new task
5. Before starting any new task, run `collab list` to check for pending messages
6. If there are messages, respond before proceeding: `collab add @sender "response" --refs <hash>`
7. If you make a change that affects shared interfaces, APIs, or files another worker depends on,
   notify them immediately: `collab add @other-worker "changed X in file Y 45 you may need to update Z"`
