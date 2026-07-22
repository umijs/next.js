use std::mem::take;

use anyhow::Result;
use bincode::{Decode, Encode};
use swc_core::{
    common::DUMMY_SP,
    ecma::ast::{Expr, KeyValueProp, ObjectLit, Prop, PropName, PropOrSpread},
    quote,
};
use turbo_tasks::{NonLocalValue, ResolvedVc, Vc, debug::ValueDebugFormat, trace::TraceRawVcs};
use turbopack_core::chunk::ChunkingContext;

use crate::{
    RuntimeExternalRequireMap,
    code_gen::{CodeGen, CodeGeneration},
    create_visitor,
    references::AstPath,
    runtime_functions::TURBOPACK_DYNAMIC_EXTERNAL_REQUIRE,
};

#[derive(
    PartialEq, Eq, TraceRawVcs, ValueDebugFormat, NonLocalValue, Debug, Hash, Encode, Decode,
)]
pub struct DynamicExternalRequire {
    path: AstPath,
    externals: ResolvedVc<RuntimeExternalRequireMap>,
}

impl DynamicExternalRequire {
    pub fn new(path: AstPath, externals: ResolvedVc<RuntimeExternalRequireMap>) -> Self {
        Self { path, externals }
    }

    pub async fn code_generation(
        &self,
        _chunking_context: Vc<Box<dyn ChunkingContext>>,
    ) -> Result<CodeGeneration> {
        let externals = self.externals.await?;
        let map = Expr::Object(ObjectLit {
            span: DUMMY_SP,
            props: externals
                .iter()
                .map(|(request, runtime_request)| {
                    PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Str(request.as_str().into()),
                        value: Box::new(Expr::Lit(runtime_request.as_str().into())),
                    })))
                })
                .collect(),
        });

        let visitor = create_visitor!(self.path, visit_mut_expr, |expr: &mut Expr| {
            let Expr::Call(call) = expr else {
                return;
            };
            let Some(argument) = call.args.first_mut() else {
                return;
            };
            let request = take(&mut *argument.expr);
            *expr = quote!(
                "$dynamic_external_require($request, $map)" as Expr,
                dynamic_external_require: Expr = TURBOPACK_DYNAMIC_EXTERNAL_REQUIRE.into(),
                request: Expr = request,
                map: Expr = map.clone()
            );
        });

        Ok(CodeGeneration::visitors(vec![visitor]))
    }
}

impl From<DynamicExternalRequire> for CodeGen {
    fn from(value: DynamicExternalRequire) -> Self {
        CodeGen::DynamicExternalRequire(value)
    }
}
