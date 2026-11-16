function run(argv) {
  const rex = Application(argv[0]);
  if (!rex.running()) return JSON.stringify({ running: false });
  const frontmost = rex.frontmost();
  const session = rex.currentSession();
  if (session === null) return JSON.stringify({ running: true, frontmost, session: null });
  return JSON.stringify({
    running: true,
    frontmost,
    session: { id: session.id(), name: session.name(), remote: session.remote() },
  });
}
