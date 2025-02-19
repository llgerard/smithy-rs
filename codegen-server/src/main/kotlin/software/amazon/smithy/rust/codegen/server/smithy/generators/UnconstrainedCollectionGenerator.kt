/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

package software.amazon.smithy.rust.codegen.server.smithy.generators

import software.amazon.smithy.model.shapes.CollectionShape
import software.amazon.smithy.model.shapes.StructureShape
import software.amazon.smithy.model.shapes.UnionShape
import software.amazon.smithy.rust.codegen.core.rustlang.RustType
import software.amazon.smithy.rust.codegen.core.rustlang.RustWriter
import software.amazon.smithy.rust.codegen.core.rustlang.rust
import software.amazon.smithy.rust.codegen.core.rustlang.rustBlock
import software.amazon.smithy.rust.codegen.core.rustlang.rustTemplate
import software.amazon.smithy.rust.codegen.core.smithy.RuntimeType
import software.amazon.smithy.rust.codegen.core.smithy.makeMaybeConstrained
import software.amazon.smithy.rust.codegen.core.smithy.module
import software.amazon.smithy.rust.codegen.server.smithy.PubCrateConstraintViolationSymbolProvider
import software.amazon.smithy.rust.codegen.server.smithy.ServerCodegenContext
import software.amazon.smithy.rust.codegen.server.smithy.UnconstrainedShapeSymbolProvider
import software.amazon.smithy.rust.codegen.server.smithy.canReachConstrainedShape
import software.amazon.smithy.rust.codegen.server.smithy.isDirectlyConstrained

/**
 * Generates a Rust type for a constrained collection shape that is able to hold values for the corresponding
 * _unconstrained_ shape. This type is a [RustType.Opaque] wrapper tuple newtype holding a `Vec`. Upon request parsing,
 * server deserializers use this type to store the incoming values without enforcing the modeled constraints. Only after
 * the full request has been parsed are constraints enforced, via the `impl TryFrom<UnconstrainedSymbol> for
 * ConstrainedSymbol`.
 *
 * This type is never exposed to the user; it is always `pub(crate)`. Only the deserializers use it.
 *
 * Consult [UnconstrainedShapeSymbolProvider] for more details and for an example.
 */
class UnconstrainedCollectionGenerator(
    val codegenContext: ServerCodegenContext,
    private val unconstrainedModuleWriter: RustWriter,
    val shape: CollectionShape,
) {
    private val model = codegenContext.model
    private val symbolProvider = codegenContext.symbolProvider
    private val unconstrainedShapeSymbolProvider = codegenContext.unconstrainedShapeSymbolProvider
    private val pubCrateConstrainedShapeSymbolProvider = codegenContext.pubCrateConstrainedShapeSymbolProvider
    private val symbol = unconstrainedShapeSymbolProvider.toSymbol(shape)
    private val name = symbol.name
    private val publicConstrainedTypes = codegenContext.settings.codegenConfig.publicConstrainedTypes
    private val constraintViolationSymbolProvider =
        with(codegenContext.constraintViolationSymbolProvider) {
            if (publicConstrainedTypes) {
                this
            } else {
                PubCrateConstraintViolationSymbolProvider(this)
            }
        }
    private val constraintViolationSymbol = constraintViolationSymbolProvider.toSymbol(shape)
    private val constrainedShapeSymbolProvider = codegenContext.constrainedShapeSymbolProvider
    private val constrainedSymbol = if (shape.isDirectlyConstrained(symbolProvider)) {
        constrainedShapeSymbolProvider.toSymbol(shape)
    } else {
        pubCrateConstrainedShapeSymbolProvider.toSymbol(shape)
    }
    private val innerShape = model.expectShape(shape.member.target)

    fun render() {
        check(shape.canReachConstrainedShape(model, symbolProvider))

        val innerShape = model.expectShape(shape.member.target)
        val innerUnconstrainedSymbol = unconstrainedShapeSymbolProvider.toSymbol(innerShape)

        unconstrainedModuleWriter.withInlineModule(symbol.module()) {
            rustTemplate(
                """
                ##[derive(Debug, Clone)]
                pub(crate) struct $name(pub(crate) std::vec::Vec<#{InnerUnconstrainedSymbol}>);

                impl From<$name> for #{MaybeConstrained} {
                    fn from(value: $name) -> Self {
                        Self::Unconstrained(value)
                    }
                }
                """,
                "InnerUnconstrainedSymbol" to innerUnconstrainedSymbol,
                "MaybeConstrained" to constrainedSymbol.makeMaybeConstrained(),
            )

            renderTryFromUnconstrainedForConstrained(this)
        }
    }

    private fun renderTryFromUnconstrainedForConstrained(writer: RustWriter) {
        writer.rustBlock("impl std::convert::TryFrom<$name> for #{T}", constrainedSymbol) {
            rust("type Error = #T;", constraintViolationSymbol)

            rustBlock("fn try_from(value: $name) -> Result<Self, Self::Error>") {
                if (innerShape.canReachConstrainedShape(model, symbolProvider)) {
                    val resolvesToNonPublicConstrainedValueType =
                        innerShape.canReachConstrainedShape(model, symbolProvider) &&
                            !innerShape.isDirectlyConstrained(symbolProvider) &&
                            innerShape !is StructureShape &&
                            innerShape !is UnionShape
                    val innerConstrainedSymbol = if (resolvesToNonPublicConstrainedValueType) {
                        pubCrateConstrainedShapeSymbolProvider.toSymbol(innerShape)
                    } else {
                        constrainedShapeSymbolProvider.toSymbol(innerShape)
                    }
                    val innerConstraintViolationSymbol = constraintViolationSymbolProvider.toSymbol(innerShape)

                    rustTemplate(
                        """
                        let res: Result<std::vec::Vec<#{InnerConstrainedSymbol}>, (usize, #{InnerConstraintViolationSymbol})> = value
                            .0
                            .into_iter()
                            .enumerate()
                            .map(|(idx, inner)| inner.try_into().map_err(|inner_violation| (idx, inner_violation)))
                            .collect();
                        let inner = res.map_err(|(idx, inner_violation)| Self::Error::Member(idx, inner_violation))?;
                        """,
                        "InnerConstrainedSymbol" to innerConstrainedSymbol,
                        "InnerConstraintViolationSymbol" to innerConstraintViolationSymbol,
                        "TryFrom" to RuntimeType.TryFrom,
                    )
                } else {
                    rust("let inner = value.0;")
                }

                if (shape.isDirectlyConstrained(symbolProvider)) {
                    rust("Self::try_from(inner)")
                } else {
                    rust("Ok(Self(inner))")
                }
            }
        }
    }
}
