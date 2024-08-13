# Hardware Requirements

The MineWars server is very efficient on resources. You can run it on pretty
much anything. Especially if you do not plan on running many simultaneous
gameplay sessions.

However, it is designed with multithreading and scalability in mind. Hence,
we recommend a computer (or virtual machine) with at least 2 CPU cores.

We focus on talking about the CPU, because other resources are unlikely
to be a limitation. MineWars uses very little RAM and network bandwidth.
If you have many users, you will almost certainly run into CPU limitations
before anything else.

## Overprovisioning

The Host server is designed to degrade gracefully in performance. When
overloaded, it will keep running just fine, but players may experience
subtle increases in latency. The server can manage many gameplay sessions
and players, but the game might start to feel less responsive during times
when there is heavy activity and the CPU is very busy.

If you care about ensuring the best performance (such as for a professional
competitive LAN tournament), we recommend one CPU core per session, or at
least 4 CPU cores, whichever is more. Extra cores cannot hurt.

If you care about hosting many users on weak hardware (such as if you'd
like to offer an online service for many players, on a budget), you will
need to decide how much degradation you consider acceptable.

Think of it this way: imagine you have 8 CPU cores. If you host 9 parallel
sessions, what are the chances that all 9 of them will need the CPU at the
exact same time? Pretty low. Now, if you have 100 (or 1000) sessions, what
are the chances? Much higher.

When it happens, users will need to wait for their turn on the CPU. The server
tries to be fair, but if these situations happen often, players will start to
feel that their inputs become less responsive, and the server might start to
miss its timer deadlines (timer-based gameplay mechanics like respawning and
construction might have their results delayed). It will worsen the gameplay
experience. When it gets bad enough that you consider it unacceptable,
it's time to upgrade your hardware. ;)
