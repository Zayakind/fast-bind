use ssh2::Session;
use std::net::TcpStream;
use crate::error::AppError;

pub struct SshConnection {
    session: Session,
}

pub fn connect_ssh(host: String, port: u16, username: String, password: String) -> Result<(), AppError> {
    let tcp = TcpStream::connect(format!("{}:{}", host, port))
        .map_err(|e| AppError::Ssh(format!("Failed to connect: {}", e)))?;
    
    let mut sess = Session::new()
        .map_err(|e| AppError::Ssh(format!("Failed to create session: {}", e)))?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .map_err(|e| AppError::Ssh(format!("Failed to handshake: {}", e)))?;
    
    sess.userauth_password(&username, &password)
        .map_err(|e| AppError::Ssh(format!("Failed to authenticate: {}", e)))?;
    
    if !sess.authenticated() {
        return Err(AppError::Ssh("Authentication failed".into()));
    }
    
    // TODO: Сохранение сессии для последующего использования
    
    Ok(())
} 