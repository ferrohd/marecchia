use async_std::io;
use async_trait::async_trait;
use libp2p::{request_response::{ProtocolName, self}, core::upgrade::{read_length_prefixed, write_length_prefixed}};
use libp2p::futures::prelude::*;

#[derive(Debug, Clone)]
pub struct SegmentExchangeProtocol();
#[derive(Clone)]
pub struct SegmentExchangeCodec();
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentRequest(pub String);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentResponse(pub Vec<u8>);

impl ProtocolName for SegmentExchangeProtocol {
    fn protocol_name(&self) -> &[u8] {
        "/file-exchange/1".as_bytes()
    }
}

#[async_trait]
impl request_response::Codec for SegmentExchangeCodec {
    type Protocol = SegmentExchangeProtocol;
    type Request = SegmentRequest;
    type Response = SegmentResponse;
    
    async fn read_request<T>(
        &mut self,
        _: &SegmentExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
    T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 1_000_000).await?;
        
        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        
        Ok(SegmentRequest(String::from_utf8(vec).unwrap()))
    }
    
    async fn read_response<T>(
        &mut self,
        _: &SegmentExchangeProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
    T: AsyncRead + Unpin + Send,
    {
        let vec = read_length_prefixed(io, 500_000_000).await?; // update transfer maximum
        
        if vec.is_empty() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        
        Ok(SegmentResponse(vec))
    }
    
    async fn write_request<T>(
        &mut self,
        _: &SegmentExchangeProtocol,
        io: &mut T,
        SegmentRequest(data): SegmentRequest,
    ) -> io::Result<()>
    where
    T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;
        
        Ok(())
    }
    
    async fn write_response<T>(
        &mut self,
        _: &SegmentExchangeProtocol,
        io: &mut T,
        SegmentResponse(data): SegmentResponse,
    ) -> io::Result<()>
    where
    T: AsyncWrite + Unpin + Send,
    {
        write_length_prefixed(io, data).await?;
        io.close().await?;
        
        Ok(())
    }
}