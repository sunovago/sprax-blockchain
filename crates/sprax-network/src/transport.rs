use crate::{
    error::NetworkError, identity::NetworkIdentity, message::NetworkMessage, peer::PeerId,
};
use snow::TransportState;
use sprax_types::Hash32;
use std::sync::Arc;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    sync::Mutex,
};

const NOISE_PATTERN: &str = "Noise_XX_25519_ChaChaPoly_BLAKE2s";
const MAX_NOISE_PLAINTEXT_CHUNK: usize = 65000;
const MAX_NOISE_RECORD_LEN: usize = 65535;

/// Established authenticated and encrypted Noise transport channel.
pub struct AuthenticatedChannel {
    pub remote_peer_id: PeerId,
    pub remote_static_key: [u8; 32],
    transport: Arc<Mutex<TransportState>>,
}

impl std::fmt::Debug for AuthenticatedChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthenticatedChannel")
            .field("remote_peer_id", &self.remote_peer_id)
            .field("remote_static_key", &hex::encode(self.remote_static_key))
            .finish()
    }
}

impl AuthenticatedChannel {
    /// Performs mutual cryptographic authentication as the connection initiator.
    pub async fn initiate<S>(
        stream: &mut S,
        identity: &NetworkIdentity,
        chain_id: &str,
        genesis: &Hash32,
    ) -> Result<Self, NetworkError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let prologue = format!("SPRX-P2P-v1:{chain_id}:{genesis}").into_bytes();
        let builder: snow::Builder = snow::Builder::new(
            NOISE_PATTERN
                .parse()
                .map_err(|e: snow::Error| NetworkError::NoiseError(e.to_string()))?,
        );
        let mut session = builder
            .local_private_key(identity.private_key())
            .map_err(|e| NetworkError::NoiseError(e.to_string()))?
            .prologue(&prologue)
            .map_err(|e| NetworkError::NoiseError(e.to_string()))?
            .build_initiator()
            .map_err(|e| NetworkError::NoiseError(e.to_string()))?;

        let mut buf = vec![0u8; 1024];
        let mut read_buf = vec![0u8; 1024];

        // Handshake Msg 1: -> e
        let len1 = session
            .write_message(&[], &mut buf)
            .map_err(|e| NetworkError::NoiseError(format!("handshake msg 1 write: {e}")))?;
        write_handshake_frame(stream, &buf[..len1]).await?;

        // Handshake Msg 2: <- e, ee, s, es
        let frame2 = read_handshake_frame(stream).await?;
        session.read_message(&frame2, &mut read_buf).map_err(|e| {
            NetworkError::AuthenticationFailed(format!("handshake msg 2 read: {e}"))
        })?;

        // Handshake Msg 3: -> s, se
        let len3 = session
            .write_message(&[], &mut buf)
            .map_err(|e| NetworkError::NoiseError(format!("handshake msg 3 write: {e}")))?;
        write_handshake_frame(stream, &buf[..len3]).await?;

        let transport = session
            .into_transport_mode()
            .map_err(|e| NetworkError::NoiseError(format!("into_transport_mode: {e}")))?;

        let remote_static = transport.get_remote_static().ok_or_else(|| {
            NetworkError::AuthenticationFailed("missing remote static key after handshake".into())
        })?;

        if remote_static.len() != 32 {
            return Err(NetworkError::AuthenticationFailed(
                "invalid remote static key length".into(),
            ));
        }

        let mut remote_static_key = [0u8; 32];
        remote_static_key.copy_from_slice(remote_static);

        if remote_static_key == identity.public_key() {
            return Err(NetworkError::SecurityViolation(
                "self-connection detected and rejected".into(),
            ));
        }

        let remote_peer_id = PeerId::from_public_key(&remote_static_key);

        Ok(Self {
            remote_peer_id,
            remote_static_key,
            transport: Arc::new(Mutex::new(transport)),
        })
    }

    /// Performs mutual cryptographic authentication as the connection responder.
    pub async fn respond<S>(
        stream: &mut S,
        identity: &NetworkIdentity,
        chain_id: &str,
        genesis: &Hash32,
    ) -> Result<Self, NetworkError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let prologue = format!("SPRX-P2P-v1:{chain_id}:{genesis}").into_bytes();
        let builder: snow::Builder = snow::Builder::new(
            NOISE_PATTERN
                .parse()
                .map_err(|e: snow::Error| NetworkError::NoiseError(e.to_string()))?,
        );
        let mut session = builder
            .local_private_key(identity.private_key())
            .map_err(|e| NetworkError::NoiseError(e.to_string()))?
            .prologue(&prologue)
            .map_err(|e| NetworkError::NoiseError(e.to_string()))?
            .build_responder()
            .map_err(|e| NetworkError::NoiseError(e.to_string()))?;

        let mut buf = vec![0u8; 1024];
        let mut read_buf = vec![0u8; 1024];

        // Handshake Msg 1: -> e
        let frame1 = read_handshake_frame(stream).await?;
        session.read_message(&frame1, &mut read_buf).map_err(|e| {
            NetworkError::AuthenticationFailed(format!("handshake msg 1 read: {e}"))
        })?;

        // Handshake Msg 2: <- e, ee, s, es
        let len2 = session
            .write_message(&[], &mut buf)
            .map_err(|e| NetworkError::NoiseError(format!("handshake msg 2 write: {e}")))?;
        write_handshake_frame(stream, &buf[..len2]).await?;

        // Handshake Msg 3: -> s, se
        let frame3 = read_handshake_frame(stream).await?;
        session.read_message(&frame3, &mut read_buf).map_err(|e| {
            NetworkError::AuthenticationFailed(format!("handshake msg 3 read: {e}"))
        })?;

        let transport = session
            .into_transport_mode()
            .map_err(|e| NetworkError::NoiseError(format!("into_transport_mode: {e}")))?;

        let remote_static = transport.get_remote_static().ok_or_else(|| {
            NetworkError::AuthenticationFailed("missing remote static key after handshake".into())
        })?;

        if remote_static.len() != 32 {
            return Err(NetworkError::AuthenticationFailed(
                "invalid remote static key length".into(),
            ));
        }

        let mut remote_static_key = [0u8; 32];
        remote_static_key.copy_from_slice(remote_static);

        if remote_static_key == identity.public_key() {
            return Err(NetworkError::SecurityViolation(
                "self-connection detected and rejected".into(),
            ));
        }

        let remote_peer_id = PeerId::from_public_key(&remote_static_key);

        Ok(Self {
            remote_peer_id,
            remote_static_key,
            transport: Arc::new(Mutex::new(transport)),
        })
    }

    /// Clones the inner transport mutex for splitting into independent read/write loops.
    #[must_use]
    pub fn transport_handle(&self) -> Arc<Mutex<TransportState>> {
        Arc::clone(&self.transport)
    }

    /// Splits the authenticated channel into an encrypted reader and writer.
    pub fn split_io<R, W>(
        &self,
        reader: R,
        writer: W,
        max_message_size: usize,
    ) -> (EncryptedReader<R>, EncryptedWriter<W>)
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let r = EncryptedReader {
            reader,
            transport: Arc::clone(&self.transport),
            decrypted_buffer: Vec::new(),
            max_message_size,
        };
        let w = EncryptedWriter {
            writer,
            transport: Arc::clone(&self.transport),
            max_message_size,
        };
        (r, w)
    }
}

/// Encrypted stream writer enforcing Noise AEAD framing.
pub struct EncryptedWriter<W> {
    writer: W,
    transport: Arc<Mutex<TransportState>>,
    max_message_size: usize,
}

impl<W> std::fmt::Debug for EncryptedWriter<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedWriter")
            .field("max_message_size", &self.max_message_size)
            .finish()
    }
}

impl<W: AsyncWrite + Unpin> EncryptedWriter<W> {
    pub async fn write_message(&mut self, msg: &NetworkMessage) -> Result<(), NetworkError> {
        let framed_bytes = msg.encode_bounded(self.max_message_size)?;
        let mut offset = 0;
        let mut cipher_buf = vec![0u8; MAX_NOISE_RECORD_LEN];

        while offset < framed_bytes.len() {
            let chunk_end = (offset + MAX_NOISE_PLAINTEXT_CHUNK).min(framed_bytes.len());
            let chunk = &framed_bytes[offset..chunk_end];

            let cipher_len = {
                let mut trans = self.transport.lock().await;
                trans
                    .write_message(chunk, &mut cipher_buf)
                    .map_err(|e| NetworkError::NoiseError(format!("encryption failed: {e}")))?
            };

            let len_bytes = (cipher_len as u16).to_be_bytes();
            self.writer
                .write_all(&len_bytes)
                .await
                .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
            self.writer
                .write_all(&cipher_buf[..cipher_len])
                .await
                .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

            offset = chunk_end;
        }

        self.writer
            .flush()
            .await
            .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }
}

/// Encrypted stream reader enforcing Noise AEAD decryption and message boundaries.
pub struct EncryptedReader<R> {
    reader: R,
    transport: Arc<Mutex<TransportState>>,
    decrypted_buffer: Vec<u8>,
    max_message_size: usize,
}

impl<R> std::fmt::Debug for EncryptedReader<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptedReader")
            .field("buffered_bytes", &self.decrypted_buffer.len())
            .field("max_message_size", &self.max_message_size)
            .finish()
    }
}

impl<R: AsyncRead + Unpin> EncryptedReader<R> {
    pub async fn read_message(&mut self) -> Result<NetworkMessage, NetworkError> {
        // First, ensure we have at least 4 bytes for the application message length
        self.ensure_decrypted_bytes(4).await?;

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&self.decrypted_buffer[..4]);
        let app_len = u32::from_be_bytes(len_bytes) as usize;

        if app_len == 0 || app_len > self.max_message_size {
            return Err(NetworkError::FrameError(format!(
                "application message length {app_len} exceeds limit {}",
                self.max_message_size
            )));
        }

        // Ensure we have the entire application message payload
        let total_required = 4 + app_len;
        self.ensure_decrypted_bytes(total_required).await?;

        let payload = &self.decrypted_buffer[4..total_required];
        let msg = NetworkMessage::decode(payload)?;

        // Drain the consumed message from buffer
        self.decrypted_buffer.drain(..total_required);
        Ok(msg)
    }

    async fn ensure_decrypted_bytes(&mut self, needed: usize) -> Result<(), NetworkError> {
        let mut cipher_buf = vec![0u8; MAX_NOISE_RECORD_LEN];
        let mut plain_buf = vec![0u8; MAX_NOISE_RECORD_LEN];

        while self.decrypted_buffer.len() < needed {
            // Read 2-byte record length
            let mut record_len_bytes = [0u8; 2];
            self.reader
                .read_exact(&mut record_len_bytes)
                .await
                .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

            let record_len = u16::from_be_bytes(record_len_bytes) as usize;
            if !(16..=MAX_NOISE_RECORD_LEN).contains(&record_len) {
                return Err(NetworkError::FrameError(format!(
                    "invalid encrypted record length {record_len}"
                )));
            }

            self.reader
                .read_exact(&mut cipher_buf[..record_len])
                .await
                .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;

            let plain_len = {
                let mut trans = self.transport.lock().await;
                trans
                    .read_message(&cipher_buf[..record_len], &mut plain_buf)
                    .map_err(|e| {
                        NetworkError::AuthenticationFailed(format!(
                            "record decryption/authentication failed: {e}"
                        ))
                    })?
            };

            self.decrypted_buffer
                .extend_from_slice(&plain_buf[..plain_len]);
        }

        Ok(())
    }
}

async fn write_handshake_frame<S: AsyncWrite + Unpin>(
    stream: &mut S,
    frame: &[u8],
) -> Result<(), NetworkError> {
    if frame.len() > 4096 {
        return Err(NetworkError::HandshakeFailed(
            "handshake message exceeds 4 KiB limit".into(),
        ));
    }
    let len_bytes = (frame.len() as u16).to_be_bytes();
    stream
        .write_all(&len_bytes)
        .await
        .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
    stream
        .write_all(frame)
        .await
        .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
    stream
        .flush()
        .await
        .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
    Ok(())
}

async fn read_handshake_frame<S: AsyncRead + Unpin>(
    stream: &mut S,
) -> Result<Vec<u8>, NetworkError> {
    let mut len_bytes = [0u8; 2];
    stream
        .read_exact(&mut len_bytes)
        .await
        .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
    let len = u16::from_be_bytes(len_bytes) as usize;
    if len == 0 || len > 4096 {
        return Err(NetworkError::HandshakeFailed(
            "invalid handshake frame length".into(),
        ));
    }
    let mut buf = vec![0u8; len];
    stream
        .read_exact(&mut buf)
        .await
        .map_err(|e| NetworkError::ConnectionFailed(e.to_string()))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authenticated_noise_channel_roundtrip() {
        let (mut client_io, mut server_io) = tokio::io::duplex(64 * 1024);
        let id_client = NetworkIdentity::generate().unwrap();
        let id_server = NetworkIdentity::generate().unwrap();
        let chain_id = "sprx-testnet-1";
        let genesis = Hash32::new([7u8; 32]);

        let id_server_clone = id_server.clone();
        let server_task = tokio::spawn(async move {
            let channel =
                AuthenticatedChannel::respond(&mut server_io, &id_server_clone, chain_id, &genesis)
                    .await?;
            Ok::<_, NetworkError>((channel, server_io))
        });

        let client_channel =
            AuthenticatedChannel::initiate(&mut client_io, &id_client, chain_id, &genesis)
                .await
                .unwrap();
        let (server_channel, server_io) = server_task.await.unwrap().unwrap();

        assert_eq!(client_channel.remote_peer_id, *id_server.peer_id());
        assert_eq!(server_channel.remote_peer_id, *id_client.peer_id());

        // Test sending message over encrypted split IO
        let (client_r, client_w) = tokio::io::split(client_io);
        let (server_r, server_w) = tokio::io::split(server_io);

        let (_cr, mut cw) = client_channel.split_io(client_r, client_w, 1024 * 1024);
        let (mut sr, _sw) = server_channel.split_io(server_r, server_w, 1024 * 1024);

        let ping_msg = NetworkMessage::Ping { nonce: 999 };
        cw.write_message(&ping_msg).await.unwrap();

        let received = sr.read_message().await.unwrap();
        assert_eq!(received, ping_msg);
    }

    #[tokio::test]
    async fn test_genesis_mismatch_fails_authentication() {
        let (mut client_io, mut server_io) = tokio::io::duplex(64 * 1024);
        let id_client = NetworkIdentity::generate().unwrap();
        let id_server = NetworkIdentity::generate().unwrap();
        let chain_id = "sprx-testnet-1";
        let genesis1 = Hash32::new([1u8; 32]);
        let genesis2 = Hash32::new([2u8; 32]);

        let client_task = tokio::spawn(async move {
            AuthenticatedChannel::initiate(&mut client_io, &id_client, chain_id, &genesis1).await
        });
        let server_task = tokio::spawn(async move {
            AuthenticatedChannel::respond(&mut server_io, &id_server, chain_id, &genesis2).await
        });
        let (client_res, server_res) = tokio::join!(client_task, server_task);

        let c_err = client_res.unwrap().is_err();
        let s_err = server_res.unwrap().is_err();
        assert!(c_err || s_err);
    }

    #[tokio::test]
    async fn test_self_connection_fails() {
        let (mut client_io, mut server_io) = tokio::io::duplex(64 * 1024);
        let id = NetworkIdentity::generate().unwrap();
        let chain_id = "sprx-testnet-1";
        let genesis = Hash32::new([1u8; 32]);

        let id_clone = id.clone();
        let client_task = tokio::spawn(async move {
            AuthenticatedChannel::initiate(&mut client_io, &id, chain_id, &genesis).await
        });
        let server_task = tokio::spawn(async move {
            AuthenticatedChannel::respond(&mut server_io, &id_clone, chain_id, &genesis).await
        });
        let (client_res, server_res) = tokio::join!(client_task, server_task);

        let c_err = client_res.unwrap().is_err();
        let s_err = server_res.unwrap().is_err();
        assert!(c_err && s_err);
    }
}
