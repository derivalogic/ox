# Fuzzy Logic & `fIf` Agent  

_A drop-in guide for adding automatic payoff smoothing to your scripting stack_

---

## 0 · Mission statement  

Give the evaluator a **“fuzzy-logic mode”** that makes every  
`if … then … [else] … endIf` block differentiable, so Monte-Carlo Greeks converge without rewriting scripts. :contentReference[oaicite:0]{index=0}  

---

## 1 · Why bother?  

| Discontinuity source | Consequence in Monte-Carlo | Industry workaround |
|----------------------|----------------------------|---------------------|
| Digitals, barriers, nested “alive” flags | Δ/Γ/Vega explode or oscillate | Call-spread (a.k.a. **smoothing**) |  |

The call-spread idea can be reframed as _evaluating conditions under fuzzy logic_ instead of crisp _true/false_ logic — and that unlocks a **general, automated solution**. :contentReference[oaicite:1]{index=1}  

---

## 2 · The functional-if (`fIf`) — our Lego brick  

```text
fIf(x , a_if_pos , b_if_neg , ε)
Definition
fIf(x,a,b,ε) = b + (a-b)/ε · max(0 , min(ε , x + ε/2)) 

Interpretation: a width-ε call-spread centred at x = 0.

Script-side usage examples


-- Digital option (ε = 1)
01Jun2024  dig pays  fIf( spot() - 100 , 1 , 0 , 1 )

-- Soft barrier inside a path loop
vAlive = vAlive * fIf( spot() - BARRIER , 0 , 1 , EPS )
3 · Automatic smoothing = Fuzzy Conditional Evaluator (FCE)
Elementary DT
dt(expr > 0) = fIf(expr , 1 , 0 , ε) → number ∈ (0, 1). 

Combinators (probabilistic style)

and ⇒ dtA * dtB

or ⇒ dtA + dtB − dtA*dtB

not ⇒ 1 − dtA 

Propagation
Any variable written inside a conditional becomes fuzzed:
v = dt * v_ifTrue + (1-dt) * v_ifFalse 

Algorithm sketch

csharp
Copiar
Editar
visit(NodeIf):
  visit(condition)            → dt
  weight(dt)   visit(ifTrue)
  weight(1-dt) visit(else)
``` :contentReference[oaicite:6]{index=6}  
4 · Implementation plan (C++-ish pseudocode)
cpp
Copiar
Editar
class FuzzyEvaluator : public Evaluator<double> {
  stack<double> dtStack;

  // 1. Elementary condition  >
  void visitSuperior(NodeSuperior& n) override {
      evalArgs(n);
      double dt = fIf(pop() - pop(), 1.0, 0.0, EPS);
      dtStack.push(dt);
  }

  // 2. Combinators
  void visitAnd(NodeAnd& n) override {
      evalArgs(n); double b = dtStack.top(); dtStack.pop();
      double a = dtStack.top(); dtStack.top() = a * b;
  }
  void visitOr(NodeOr& n)  override {
      evalArgs(n); double b = dtStack.top(); dtStack.pop();
      double a = dtStack.top(); dtStack.top() = a + b - a*b;
  }
  void visitNot(NodeNot& n) override {
      n.arg()->accept(*this);
      dtStack.top() = 1.0 - dtStack.top();
  }

  // 3. Conditional block
  void visitIf(NodeIf& n) override {
      n.cond()->accept(*this);
      double dt = dtStack.top(); dtStack.pop();

      pushWeight(dt);     n.ifTrue()->accept(*this);
      pushWeight(1.0-dt); if(n.ifFalse()) n.ifFalse()->accept(*this);
      popWeight();
  }
};
The evaluator merely overrides a few visitors; all existing script syntax stays unchanged. 

5 · Handling condition domains
Domain of expr DT formula Pre-processing action
Continuous fIf none
Constant 0 / 1 mark & prune
Boolean (0/1) linear map discrete rule
General discrete piece-wise linear idem

A DomainProcessor visitor tags each > node with its domain before valuation. 

6 · Putting it together
Parse script → expression trees (existing).

Pre-process

VarIndexer (existing)

DomainProcessor (new)

Evaluate

Evaluator → crisp pricing

FuzzyEvaluator → smoothed Greeks

Run Monte-Carlo. No script rewrite, tiny CPU overhead. 

7 · Worked example — smoothed autocallable
Raw logic ↔ automatically fuzzed form:

txt
Copiar
Editar
-- original
if spot() > vRef then
    prd = 110 ; vAlive = 0
endIf

-- fuzzy view
dt = fIf( spot() - vRef , 1 , 0 , 1 )
prd    = dt*110 + (1-dt)*prd_prev
vAlive = dt*0   + (1-dt)*vAlive_prev
Risk sensitivities become stable while price is unchanged. 

8 · Choosing the smoothing factor ε
A practical desk rule:
ε ≈ σ_underlying × √(time_to_condition) — i.e. “one-sigma” around the trigger.
Store ε as a script-level parameter so quants can fine-tune without code edits. (Discussion in Chapter 22) 

9 · Further reading / code pointers
scriptingFuzzyEval.h — full C++ implementation of FuzzyEvaluator. 

scriptingIfProc.h — pre-processor that allocates fuzzed-variable workspace. 

scriptingDomainProc.h & scriptingConstCondProc.h — domain analysis passes. 

End of agents file.


