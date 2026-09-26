#!/usr/bin/env python3
"""play.py — tkinter UI vs the JS bot (via bridge.js stdio). Run: python3 play.py"""
import json, queue, subprocess, sys, threading, tkinter as tk

N, SZ, M = 19, 720, 40
STEP = (SZ - 2 * M) / (N - 1)
proc = subprocess.Popen([sys.executable if False else "node", "bridge.js"],
                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                        text=True, bufsize=1, cwd="/home/genius74o/game")
resp_q = queue.Queue()
threading.Thread(target=lambda: [resp_q.put(json.loads(l)) for l in proc.stdout],
                 daemon=True).start()

def ask(msg):
    proc.stdin.write(json.dumps(msg) + "\n"); proc.stdin.flush()
    return resp_q.get()

def X(x): return M + x * STEP
def Y(y): return M + (N - 1 - y) * STEP

root = tk.Tk(); root.title("Enclosure — you vs bot")
top = tk.Label(root, font=("sans", 13)); top.pack()
cv = tk.Canvas(root, width=SZ, height=SZ, bg="#0b1220"); cv.pack()
status = tk.Label(root, font=("sans", 12)); status.pack()
btns = tk.Frame(root); btns.pack()
state, sel, targets = None, None, []

def draw():
    cv.delete("all")
    for i in range(N):
        cv.create_line(X(i), Y(0), X(i), Y(18), fill="#1e293b")
        cv.create_line(X(0), Y(i), X(18), Y(i), fill="#1e293b")
    cols = {"blue": "#60a5fa", "red": "#f87171"}
    for c in ("blue", "red"):
        for s in state["segments"][c]:
            cv.create_line(X(s["from"][0]), Y(s["from"][1]), X(s["to"][0]), Y(s["to"][1]),
                           fill=cols[c], width=5 if s["invincible"] else 2)
        for n in state["nodes"][c]:
            r = 9 if sel == n else 6
            cv.create_oval(X(n[0])-r, Y(n[1])-r, X(n[0])+r, Y(n[1])+r,
                           fill="#2563eb" if c == "blue" else "#dc2626",
                           outline="white", tags=(f"n:{c}:{n[0]},{n[1]}",))
    for t in targets:
        cv.create_oval(X(t[0])-6, Y(t[1])-6, X(t[0])+6, Y(t[1])+6,
                       outline="#22c55e", width=2, tags=(f"t:{t[0]},{t[1]}",))
    top.config(text=f"Blue {state['scores']['blue']:.1f} (area {state['areas']['blue']:.1f})   "
                    f"move {state['moveNumber']}/120 · {state['turn']} ({state['actionsRemaining']} left)   "
                    f"Red {state['scores']['red']:.1f} (area {state['areas']['red']:.1f})")

def apply_reply(j):
    global state, sel, targets
    if "state" in j and j["state"]: state = j["state"]
    sel, targets = None, []
    if j.get("error"): status.config(text="Illegal: " + j["error"])
    elif state["over"]:
        w = state["winner"]; status.config(text="Game over — " + ("draw" if w == "draw" else w + " wins"))
    elif j.get("bot", {}).get("moves"):
        b = j["bot"]; ms = [f"{m['from']}->{m['to']}" for m in b["moves"]]
        status.config(text=f"Bot ({b['ms']}ms): " + " · ".join(ms) + ". Your move.")
    else: status.config(text=f"Your move ({state['human']}). Click a dot, then a green target.")
    draw()

def busy(fn):
    status.config(text="Bot thinking…"); root.update()
    threading.Thread(target=lambda: root.after(0, lambda: apply_reply(fn())), daemon=True).start()

def new_game(color): busy(lambda: ask({"cmd": "new", "color": color}))
def do_pass(): busy(lambda: ask({"cmd": "human", "pass": True}))

def click(e):
    global sel, targets
    if not state or state["over"] or state["turn"] != state["human"]: return
    it = cv.find_closest(e.x, e.y)
    if not it: return
    tags = cv.gettags(it[0])
    for tg in tags:
        if tg.startswith("t:"):
            to = [int(v) for v in tg[2:].split(",")]
            busy(lambda: ask({"cmd": "human", "from": sel, "to": to})); return
        if tg.startswith("n:"):
            _, c, p = tg.split(":"); p = [int(v) for v in p.split(",")]
            if c != state["human"]: return
            sel = p
            status.config(text=f"From {p} — checking legal targets…"); root.update()
            r = ask({"cmd": "legal", "from": p}); targets = r.get("targets", [])
            status.config(text=f"From {p} — {len(targets)} legal targets." if targets else "No legal moves there — pick another dot.")
            draw(); return
    sel, targets = None, []; draw()

cv.bind("<Button-1>", click)
tk.Button(btns, text="New as Blue", command=lambda: new_game("blue")).pack(side="left")
tk.Button(btns, text="New as Red", command=lambda: new_game("red")).pack(side="left")
tk.Button(btns, text="Pass", command=do_pass).pack(side="left")
apply_reply(ask({"cmd": "new", "color": "blue"}))
root.mainloop()
