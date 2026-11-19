//! Canonicalização JSON JCS (RFC 8785) — **subset honesto** do Radiant Reg.
//!
//! O que o subset implementa (e o que NÃO):
//! - **Ordenação de chaves por unidades de código UTF-16** (RFC 8785 §3.2.3) —
//!   idêntica ao JSON.stringify do ECMAScript. ATENÇÃO: `&str` compara por
//!   **pontos de código** (escalar Unicode), e `serde_json::Map` (sem
//!   `preserve_order`) é um `BTreeMap` que itera nessa ordem. As duas ordens
//!   divergem quando há chaves astrais (fora do BMP): U+1F600 "😀" tem
//!   primeiro surrogate UTF-16 0xD83D, que é MENOR que 0xFB33 (דּ) e que
//!   0xFFFD — sob UTF-16 "😀" vem ANTES de דּ e de U+FFFD; por pontos de
//!   código (0x1F600) viria DEPOIS de ambos. Como o RFC 8785 manda ordenar
//!   por UTF-16, ordenamos explicitamente por `encode_utf16()` e PINAMOS a
//!   divergência no teste `ordenacao_utf16_nao_code_point` — nunca confiar
//!   no BTreeMap.
//! - **Escapes de string ECMAScript** (RFC 8785 §3.2.2.2): apenas `\b \t \n
//!   \f \r \" \\` e controles < 0x20 como `\u00xx` com hex MINÚSCULO;
//!   todo o resto é UTF-8 cru. Rust `&str` é UTF-8 válido por construção,
//!   então o caso "lone surrogate" do ES2019 é impossível aqui (não há
//!   surrogate solitário num `char`).
//! - **Números inteiros sem ponto**: `Value::Number` inteiro é escrito como
//!   decimal puro (`as_i128`), nunca `4.0` nem `1e+30`. A faixa carregável
//!   por `serde_json::Value` (sem `arbitrary_precision`) é i64::MIN..=u64::MAX
//!   — para Centavos (i128) além dessa faixa use [`numero_inteiro`] (que
//!   recusa com erro tipado) ou carregue como string decimal.
//!   Nota honesta: `serde_json::to_value(i128)` produz **string** (serde_json
//!   1.0.154, ser.rs:918-928 serializa i128 como `Value::String` no Value) —
//!   determinístico e estável, mas é string, não número.
//! - **f64 = erro tipado** [`JcsError::JcsFloatProibido`] — doutrina
//!   zero-float (docs/blueprint/20-ARQUITETURA-RUST.md §4.5). Banido mesmo
//!   integral (`4.0`): float na cadeia significa que a exatidão já se perdeu
//!   a montante. O RFC 8785 pleno tem o formato de número ES6 (parte mais
//!   difícil da spec) — aqui nem existe, por isso SUBSET.
//! - Surrogate pairs não escapados, `\u2028`/`\u2029` crus (JSON.stringify
//!   também não escapa), `/` cru — igual ao ECMAScript.

use serde_json::{Number, Value};
use thiserror::Error;

/// Erro da canonicalização JCS — tipado e auditável (provenance, ADR-0010).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum JcsError {
    /// Doutrina zero-float (20-ARQUITETURA-RUST.md §4.5): um `f64` chegou ao
    /// ledger. O RFC 8785 pleno formata float via ES6 number-to-string; aqui
    /// float nem entra — o chamador corrigiu o tipo a montante.
    #[error(
        "JcsFloatProibido: número de ponto flutuante em {path} — o ledger só aceita inteiros exatos (Centavos/i128)"
    )]
    JcsFloatProibido {
        /// Caminho do valor no documento, ex. `$.metadata.total` ou `$.itens[2]`.
        path: String,
    },
    /// Inteiro fora da faixa de `serde_json::Number` (i64::MIN..=u64::MAX
    /// sem `arbitrary_precision`). Carregue como string decimal se precisar
    /// do valor completo.
    #[error(
        "inteiro {valor} fora da faixa de serde_json::Number (i64/u64); carregue como string decimal"
    )]
    InteiroForaDeFaixa {
        /// O valor recusado, para o erro ser auditável.
        valor: i128,
    },
}

/// Constrói um `serde_json::Number` a partir de i128 (Centavos) — erro tipado
/// se fora da faixa carregável por `Value` (i64/u64). É a ponte explícita
/// "inteiro i128 sem ponto" para quem monta metadados do ledger à mão.
pub fn numero_inteiro(valor: i128) -> Result<Number, JcsError> {
    Number::from_i128(valor).ok_or(JcsError::InteiroForaDeFaixa { valor })
}

/// Canonicaliza um `serde_json::Value` conforme JCS RFC 8785 (subset).
///
/// Garantias: chaves ordenadas por UTF-16, escapes ECMAScript, inteiros sem
/// ponto, zero espaço em branco. Idempotente: `canonicalizar(parse(canon(v)))`
/// == `canon(v)` (pinnado em teste).
pub fn canonicalizar(value: &Value) -> Result<String, JcsError> {
    let mut out = String::new();
    escrever(&mut out, value, "$")?;
    Ok(out)
}

/// Canonicaliza e devolve os bytes — o insumo direto da assinatura Ed25519
/// (crate::sign assina EXATAMENTE estes bytes).
pub fn canonicalizar_para_bytes(value: &Value) -> Result<Vec<u8>, JcsError> {
    canonicalizar(value).map(String::into_bytes)
}

fn escrever(out: &mut String, value: &Value, path: &str) -> Result<(), JcsError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => escrever_numero(out, n, path)?,
        Value::String(s) => escrever_string(out, s),
        Value::Array(itens) => {
            out.push('[');
            for (i, item) in itens.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                escrever(out, item, &format!("{path}[{i}]"))?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            // RFC 8785 §3.2.3: ordem por unidades de código UTF-16 — NÃO a
            // ordem de pontos de código do BTreeMap (&str). Ver doc do módulo.
            let mut pares: Vec<(&str, &Value)> = map.iter().map(|(k, v)| (k.as_str(), v)).collect();
            pares.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
            out.push('{');
            for (i, (k, v)) in pares.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                escrever_string(out, k);
                out.push(':');
                escrever(out, v, &format!("{path}.{k}"))?;
            }
            out.push('}');
        }
    }
    Ok(())
}

fn escrever_numero(out: &mut String, n: &Number, path: &str) -> Result<(), JcsError> {
    match n.as_i128() {
        // Inteiro (i64/u64) → decimal puro, sem ponto, sem expoente.
        Some(inteiro) => out.push_str(&inteiro.to_string()),
        // Float de qualquer espécie (mesmo integral 4.0) → banido.
        None => return Err(JcsError::JcsFloatProibido { path: path.into() }),
    }
    Ok(())
}

fn escrever_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{09}' => out.push_str("\\t"),
            '\u{0A}' => out.push_str("\\n"),
            '\u{0C}' => out.push_str("\\f"),
            '\u{0D}' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // testes: unwrap é o idiom de assert
mod tests {
    use super::*;
    use serde_json::json;

    /// Vetor de strings do RFC 8785 §3.2.2.2 (adaptado): €, $, U+000F, LF,
    /// A, ', B, ", \, \, " e / — escapes ECMAScript com hex minúsculo.
    #[test]
    fn vetor_rfc_8785_escapes_de_string() {
        let conteudo = "€$\u{000F}\nA'B\"\\\\\"/";
        let v = Value::String(conteudo.into());
        let esperado = r#""€$\u000f\nA'B\"\\\\\"/""#;
        assert_eq!(canonicalizar(&v).unwrap(), esperado);
    }

    /// Todos os controles ASCII < 0x20: os cinco nomeados do ECMAScript e os
    /// 25 restantes como \u00xx minúsculo (RFC 8785 §3.2.2.2).
    #[test]
    fn controles_ascii_escapados_ecmascript() {
        let mut controle = String::new();
        for b in 0x00u8..0x20 {
            controle.push(char::from(b));
        }
        let v = Value::String(controle);
        let esperado = concat!(
            r#""\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007\b\t\n\u000b\f\r"#,
            r#"\u000e\u000f\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"#,
            r#"\u0018\u0019\u001a\u001b\u001c\u001d\u001e\u001f""#
        );
        assert_eq!(canonicalizar(&v).unwrap(), esperado);
    }

    /// RFC 8785 §3.2.3 adaptado: a ordem é por UTF-16, e a ordem de
    /// code-points do `&str`/BTreeMap diverge — o emoji 😀 (U+1F600, UTF-16
    /// 0xD83D..) vem ANTES de U+FFFD no JCS, mas DEPOIS no BTreeMap.
    #[test]
    fn ordenacao_utf16_nao_code_point() {
        let mut v = serde_json::Map::new();
        v.insert("\u{20ac}".into(), json!(2)); // 0x20AC
        v.insert("\u{00fc}".into(), json!(1)); // 0x00FC
        v.insert("\u{4e16}".into(), json!(3)); // 0x4E16
        v.insert("\u{fb33}".into(), json!(4)); // 0xFB33
        v.insert("\u{fffd}".into(), json!(6)); // 0xFFFD
        v.insert("\u{1F600}".into(), json!(5)); // astral: UTF-16 0xD83D 0xDE00
        let v = Value::Object(v);

        // Prova de relevância: o BTreeMap do serde_json itera por code-point
        // e colocaria o emoji POR ÚLTIMO — se o JCS confiasse nele, erraria.
        let ordem_btreemap: Vec<&String> = v.as_object().unwrap().keys().collect();
        assert_eq!(ordem_btreemap.last().unwrap().as_str(), "\u{1F600}");

        // Ordem UTF-16: 0x00FC < 0x20AC < 0x4E16 < 0xD83D(😀) < 0xFB33 < 0xFFFD.
        // O primeiro surrogate do emoji (0xD83D) é MENOR que דּ e que U+FFFD —
        // é a demonstração do RFC de que UTF-16 ≠ code-points.
        let esperado = "{\"ü\":1,\"€\":2,\"世\":3,\"😀\":5,\"דּ\":4,\"\u{fffd}\":6}";
        assert_eq!(canonicalizar(&v).unwrap(), esperado);
    }

    /// Números inteiros sem ponto — vetor numérico ADAPTADO do RFC 8785
    /// (os números originais são floats e aqui são banidos): 0, negativo,
    /// limites i64/u64, tudo decimal puro.
    #[test]
    fn inteiros_sem_ponto_limites() {
        let v = json!({
            "b": [-1, 0, 1],
            "a": [-9223372036854775808i64, 18446744073709551615u64, 333333333]
        });
        let esperado =
            "{\"a\":[-9223372036854775808,18446744073709551615,333333333],\"b\":[-1,0,1]}";
        assert_eq!(canonicalizar(&v).unwrap(), esperado);
    }

    /// Vetor estrutural do RFC 8785 (Appendix B) adaptado para o subset:
    /// literais + inteiros + a string oficial do vetor, ordenado por chave.
    #[test]
    fn vetor_rfc_8785_estrutural_adaptado() {
        let v = json!({
            "string": "€$\u{000F}\nA'B\"\\\\\"/",
            "inteiros": [333333333, 4, 2, 0],
            "literals": [null, true, false],
        });
        let esperado = concat!(
            r#"{"inteiros":[333333333,4,2,0],"#,
            r#""literals":[null,true,false],"#,
            r#""string":"€$\u000f\nA'B\"\\\\\"/"}"#
        );
        assert_eq!(canonicalizar(&v).unwrap(), esperado);
    }

    /// Doutrina zero-float: QUALQUER f64 (inclusive integral 4.0) é
    /// JcsError::JcsFloatProibido, com caminho do valor no documento.
    #[test]
    fn float_proibido_mesmo_integral_com_caminho() {
        let raiz = Value::from(1.5);
        let err = canonicalizar(&raiz).unwrap_err();
        assert_eq!(err, JcsError::JcsFloatProibido { path: "$".into() });
        assert!(err.to_string().contains("JcsFloatProibido"), "{err}");

        let aninhado = json!({"metadata": {"itens": [{"ok": 1}, {"total": 4.0}]}});
        let err = canonicalizar(&aninhado).unwrap_err();
        assert_eq!(
            err,
            JcsError::JcsFloatProibido {
                path: "$.metadata.itens[1].total".into()
            }
        );
    }

    /// `numero_inteiro`: i128 na faixa vira Number; além dela é erro tipado.
    #[test]
    fn numero_inteiro_i128_faixa() {
        assert_eq!(numero_inteiro(123_456).unwrap().to_string(), "123456");
        assert_eq!(
            numero_inteiro(u64::MAX as i128).unwrap().as_i128(),
            Some(u64::MAX as i128)
        );
        let err = numero_inteiro(i128::MAX).unwrap_err();
        assert!(matches!(err, JcsError::InteiroForaDeFaixa { valor } if valor == i128::MAX));
    }

    /// Idempotência: canonicalizar(parse(canon(v))) == canon(v).
    #[test]
    fn canonicalizacao_eh_idempotente() {
        let v = json!({"z": [1, 2, {"b": null, "a": true}], "a": "€\n"});
        let canon = canonicalizar(&v).unwrap();
        let reparse: Value = serde_json::from_str(&canon).unwrap();
        assert_eq!(canonicalizar(&reparse).unwrap(), canon);
    }

    /// `canonicalizar_para_bytes` devolve os mesmos bytes da string — é o
    /// insumo da assinatura (crate::sign), byte-identidade pinnada.
    #[test]
    fn bytes_identicos_a_string() {
        let v = json!({"a": 1});
        let bytes = canonicalizar_para_bytes(&v).unwrap();
        assert_eq!(bytes, canonicalizar(&v).unwrap().into_bytes());
    }

    /// Doctest de referência do crate: canonicaliza {"b":2,"a":1} → {"a":1,"b":2}.
    #[test]
    fn exemplo_do_lib_rs() {
        let v: Value = serde_json::from_str(r#"{"b":2,"a":1}"#).unwrap();
        assert_eq!(canonicalizar(&v).unwrap(), r#"{"a":1,"b":2}"#);
    }
}
