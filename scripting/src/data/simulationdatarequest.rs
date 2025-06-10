use rustatlas::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DiscountFactorRequest {
    curve: String,
    to_date: Date,
    from_date: Date,
}

impl DiscountFactorRequest {
    pub fn new(curve: String, to_date: Date, from_date: Date) -> DiscountFactorRequest {
        DiscountFactorRequest {
            curve,
            to_date,
            from_date,
        }
    }

    pub fn curve(&self) -> &String {
        &self.curve
    }

    pub fn to_date(&self) -> Date {
        self.to_date
    }

    pub fn from_date(&self) -> Date {
        self.from_date
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForwardRateRequest {
    curve: String,
    fixing_date: Date,
    start_date: Date,
    end_date: Date,
    compounding: Compounding,
    frequency: Frequency,
}

impl ForwardRateRequest {
    pub fn new(
        curve: String,
        fixing_date: Date,
        start_date: Date,
        end_date: Date,
        compounding: Compounding,
        frequency: Frequency,
    ) -> ForwardRateRequest {
        ForwardRateRequest {
            curve,
            fixing_date,
            start_date,
            end_date,
            compounding,
            frequency,
        }
    }

    pub fn curve(&self) -> &String {
        &self.curve
    }

    pub fn start_date(&self) -> Date {
        self.start_date
    }

    pub fn end_date(&self) -> Date {
        self.end_date
    }

    pub fn compounding(&self) -> Compounding {
        self.compounding
    }

    pub fn frequency(&self) -> Frequency {
        self.frequency
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExchangeRateRequest {
    first_ccy: Currency,
    second_ccy: Currency,
    date: Date,
}

impl ExchangeRateRequest {
    pub fn new(first_ccy: Currency, second_ccy: Currency, date: Date) -> ExchangeRateRequest {
        ExchangeRateRequest {
            first_ccy,
            second_ccy,
            date,
        }
    }

    pub fn first_currency(&self) -> Currency {
        self.first_ccy
    }

    pub fn second_currency(&self) -> Currency {
        self.second_ccy
    }

    pub fn date(&self) -> Date {
        self.date
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EquityRequest {
    equity_id: String,
    date: Date,
}

impl EquityRequest {
    pub fn new(equity_id: String, date: Date) -> EquityRequest {
        EquityRequest { equity_id, date }
    }

    pub fn equity_id(&self) -> &String {
        &self.equity_id
    }

    pub fn date(&self) -> Date {
        self.date
    }
}

/// # ScriptingMarketRequest
/// Meta data for market data in scripting. Holds all the meta data required to fetch the market
/// data in a scripting context.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationDataRequest {
    dfs: Vec<DiscountFactorRequest>,
    fwds: Vec<ForwardRateRequest>,
    fxs: Vec<ExchangeRateRequest>,
    equities: Vec<EquityRequest>,
}

impl SimulationDataRequest {
    pub fn new() -> SimulationDataRequest {
        SimulationDataRequest {
            dfs: Vec::new(),
            fwds: Vec::new(),
            fxs: Vec::new(),
            equities: Vec::new(),
        }
    }

    pub fn with_capacity(
        dfs_cap: usize,
        fwds_cap: usize,
        fxs_cap: usize,
        equities_cap: usize,
    ) -> SimulationDataRequest {
        SimulationDataRequest {
            dfs: Vec::with_capacity(dfs_cap),
            fwds: Vec::with_capacity(fwds_cap),
            fxs: Vec::with_capacity(fxs_cap),
            equities: Vec::with_capacity(equities_cap),
        }
    }

    pub fn push_df(&mut self, df: DiscountFactorRequest) {
        self.dfs.push(df);
    }

    pub fn push_fwd(&mut self, fwd: ForwardRateRequest) {
        self.fwds.push(fwd);
    }

    pub fn push_fx(&mut self, fx: ExchangeRateRequest) {
        self.fxs.push(fx);
    }

    pub fn push_equity(&mut self, equity: EquityRequest) {
        self.equities.push(equity);
    }

    pub fn dfs(&self) -> &Vec<DiscountFactorRequest> {
        &self.dfs
    }

    pub fn fwds(&self) -> &Vec<ForwardRateRequest> {
        &self.fwds
    }

    pub fn fxs(&self) -> &Vec<ExchangeRateRequest> {
        &self.fxs
    }

    pub fn equities(&self) -> &Vec<EquityRequest> {
        &self.equities
    }
}
