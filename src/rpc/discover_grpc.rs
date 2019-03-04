// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Discover {
    fn discover_peer(&self, o: ::grpc::RequestOptions, p: super::discover::NewPeer) -> ::grpc::SingleResponse<super::discover::Ok>;
}

// client

pub struct DiscoverClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_DiscoverPeer: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::discover::NewPeer, super::discover::Ok>>,
}

impl ::grpc::ClientStub for DiscoverClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        DiscoverClient {
            grpc_client: grpc_client,
            method_DiscoverPeer: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/Discover/DiscoverPeer".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Discover for DiscoverClient {
    fn discover_peer(&self, o: ::grpc::RequestOptions, p: super::discover::NewPeer) -> ::grpc::SingleResponse<super::discover::Ok> {
        self.grpc_client.call_unary(o, p, self.method_DiscoverPeer.clone())
    }
}

// server

pub struct DiscoverServer;


impl DiscoverServer {
    pub fn new_service_def<H : Discover + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/Discover",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/Discover/DiscoverPeer".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.discover_peer(o, p))
                    },
                ),
            ],
        )
    }
}
