use std::fs::{self, DirBuilder, File, Permissions, TryLockError};
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};

pub(crate) struct SocketClaim {
    pub(crate) listener: UnixListener,
    socket: PathBuf,
    _instance_lock: File,
}

pub(crate) enum ClaimRefusal {
    AlreadyClaimed,
    Io(io::Error),
}

impl From<io::Error> for ClaimRefusal {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl SocketClaim {
    pub(crate) fn take(socket: &Path) -> Result<Self, ClaimRefusal> {
        let folder = socket.parent().unwrap_or(Path::new("."));
        DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(folder)?;
        fs::set_permissions(folder, Permissions::from_mode(0o700))?;
        let instance_lock = File::create(socket.with_extension("lock"))?;
        match instance_lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Err(ClaimRefusal::AlreadyClaimed),
            Err(TryLockError::Error(error)) => return Err(ClaimRefusal::Io(error)),
        }
        remove_stale(socket)?;
        let listener = UnixListener::bind(socket)?;
        fs::set_permissions(socket, Permissions::from_mode(0o600))?;
        Ok(Self {
            listener,
            socket: socket.to_path_buf(),
            _instance_lock: instance_lock,
        })
    }
}

fn remove_stale(socket: &Path) -> io::Result<()> {
    match fs::remove_file(socket) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

impl Drop for SocketClaim {
    fn drop(&mut self) {
        let _ = remove_stale(&self.socket);
    }
}
