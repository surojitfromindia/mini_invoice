#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoaNodeKind {
    Group,
    Type,
    Account,
}

#[derive(Debug, Clone)]
pub struct CoaAccountSeed {
    pub key: &'static str,
    pub code: String,
    pub name_primary: &'static str,
    pub name_secondary: Option<&'static str>,
    pub description: Option<&'static str>,
    pub kind: CoaNodeKind,
    pub parent_key: Option<&'static str>,
    pub account_group_key: Option<&'static str>,
    pub account_type_key: Option<&'static str>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
}

#[derive(Debug, Clone)]
pub struct CoaTemplateSeed {
    pub key: &'static str,
    pub name_primary: &'static str,
    pub name_secondary: Option<&'static str>,
    pub description: Option<&'static str>,
    pub country_iso_code: Option<&'static str>,
    pub accounting_standard: Option<&'static str>,
    pub is_default: bool,
    pub accounts: Vec<CoaAccountSeed>,
}

pub fn template_for_country(_country_iso_code: &str) -> CoaTemplateSeed {
    default_template()
}

pub fn default_template() -> CoaTemplateSeed {
    CoaTemplateSeed {
        key: "DEFAULT_TEMPLATE",
        name_primary: "Standard Chart of Accounts",
        name_secondary: Some("Default COA"),
        description: Some("Common starter chart of accounts for a new organization"),
        country_iso_code: None,
        accounting_standard: None,
        is_default: true,
        accounts: build_default_accounts(),
    }
}

fn build_default_accounts() -> Vec<CoaAccountSeed> {
    let mut seeds = Vec::with_capacity(7 + 20 + 300);
    let mut next_leaf_code = 1001u32;

    add_group(
        &mut seeds,
        "1",
        "ASSET",
        "Assets",
        Some("Resource accounts owned or controlled by the organization"),
    );
    add_group(
        &mut seeds,
        "2",
        "LIABILITY",
        "Liabilities",
        Some("Obligations payable to external parties"),
    );
    add_group(
        &mut seeds,
        "3",
        "EQUITY",
        "Equity",
        Some("Owner and shareholder interest in the organization"),
    );
    add_group(
        &mut seeds,
        "4",
        "REVENUE",
        "Revenue",
        Some("Operating income from the ordinary course of business"),
    );
    add_group(
        &mut seeds,
        "5",
        "COGS",
        "Cost of Sales",
        Some("Direct cost linked to delivering goods or services"),
    );
    add_group(
        &mut seeds,
        "6",
        "EXPENSE",
        "Expenses",
        Some("Operating and administrative costs"),
    );
    add_group(
        &mut seeds,
        "7",
        "OTHER_INCOME",
        "Other Income",
        Some("Non-operating gains and ancillary income"),
    );

    add_type(
        &mut seeds,
        "10",
        "ASSET_CURRENT",
        "Current Assets",
        Some("Cash and near-cash resources"),
        "ASSET",
    );
    add_type(
        &mut seeds,
        "12",
        "ASSET_NON_CURRENT",
        "Non-current Assets",
        Some("Long-term resources used by the business"),
        "ASSET",
    );
    add_type(
        &mut seeds,
        "13",
        "ASSET_OTHER",
        "Other Assets",
        Some("Non-standard asset balances and long-term receivables"),
        "ASSET",
    );
    add_type(
        &mut seeds,
        "14",
        "ASSET_CONTRA",
        "Contra Assets",
        Some("Offsetting balances that reduce asset carrying value"),
        "ASSET",
    );

    add_type(
        &mut seeds,
        "21",
        "LIABILITY_CURRENT",
        "Current Liabilities",
        Some("Obligations due within one year"),
        "LIABILITY",
    );
    add_type(
        &mut seeds,
        "22",
        "LIABILITY_NON_CURRENT",
        "Non-current Liabilities",
        Some("Long-term obligations beyond one year"),
        "LIABILITY",
    );
    add_type(
        &mut seeds,
        "23",
        "LIABILITY_OTHER",
        "Other Liabilities",
        Some("Provision, statutory, and miscellaneous payables"),
        "LIABILITY",
    );

    add_type(
        &mut seeds,
        "31",
        "EQUITY_CAPITAL",
        "Capital",
        Some("Paid-in capital and owner investment accounts"),
        "EQUITY",
    );
    add_type(
        &mut seeds,
        "32",
        "EQUITY_RESERVE",
        "Reserves",
        Some("Statutory, revaluation, and legal reserve balances"),
        "EQUITY",
    );
    add_type(
        &mut seeds,
        "33",
        "EQUITY_RETAINED",
        "Retained Earnings",
        Some("Accumulated profit, loss, and distribution balances"),
        "EQUITY",
    );

    add_type(
        &mut seeds,
        "41",
        "REVENUE_PRODUCT",
        "Product Revenue",
        Some("Revenue from sale of goods"),
        "REVENUE",
    );
    add_type(
        &mut seeds,
        "42",
        "REVENUE_SERVICE",
        "Service Revenue",
        Some("Revenue from services and subscriptions"),
        "REVENUE",
    );
    add_type(
        &mut seeds,
        "43",
        "REVENUE_OTHER",
        "Other Revenue",
        Some("Ancillary operating income"),
        "REVENUE",
    );

    add_type(
        &mut seeds,
        "51",
        "COGS_DIRECT",
        "Direct Materials",
        Some("Direct cost of goods sold and raw material use"),
        "COGS",
    );
    add_type(
        &mut seeds,
        "52",
        "COGS_OVERHEAD",
        "Production Overheads",
        Some("Factory and manufacturing overhead costs"),
        "COGS",
    );
    add_type(
        &mut seeds,
        "53",
        "COGS_LOGISTICS",
        "Logistics and Duties",
        Some("Inbound freight, customs, and logistics cost"),
        "COGS",
    );

    add_type(
        &mut seeds,
        "61",
        "EXPENSE_ADMIN",
        "Administrative Expenses",
        Some("General administration and office overhead"),
        "EXPENSE",
    );
    add_type(
        &mut seeds,
        "62",
        "EXPENSE_SALES",
        "Selling and Marketing",
        Some("Sales team and marketing spend"),
        "EXPENSE",
    );
    add_type(
        &mut seeds,
        "63",
        "EXPENSE_PAYROLL",
        "Payroll and Benefits",
        Some("Employee compensation and benefits"),
        "EXPENSE",
    );
    add_type(
        &mut seeds,
        "64",
        "EXPENSE_FINANCE",
        "Finance and Banking",
        Some("Borrowing costs and bank service charges"),
        "EXPENSE",
    );
    add_type(
        &mut seeds,
        "65",
        "EXPENSE_OTHER",
        "Other Operating Expenses",
        Some("Miscellaneous operating and non-operating costs"),
        "EXPENSE",
    );

    add_type(
        &mut seeds,
        "71",
        "OTHER_INCOME_NON_OPERATING",
        "Non-operating Income",
        Some("Interest, gains, grants, and other non-operating income"),
        "OTHER_INCOME",
    );

    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "ASSET",
        "ASSET_CURRENT",
        &[
            ("ASSET_CURRENT_CASH_IN_HAND", "Cash in hand"),
            ("ASSET_CURRENT_PETTY_CASH", "Petty cash"),
            ("ASSET_CURRENT_BANK_CURRENT_ACCOUNTS", "Bank current accounts"),
            ("ASSET_CURRENT_BANK_SAVINGS_ACCOUNTS", "Bank savings accounts"),
            ("ASSET_CURRENT_ACCOUNTS_RECEIVABLE", "Accounts receivable"),
            ("ASSET_CURRENT_TRADE_RECEIVABLES", "Trade receivables"),
            ("ASSET_CURRENT_BILLS_RECEIVABLE", "Bills receivable"),
            ("ASSET_CURRENT_INVENTORY", "Inventory"),
            ("ASSET_CURRENT_WORK_IN_PROGRESS", "Work in progress"),
            ("ASSET_CURRENT_PREPAID_EXPENSES", "Prepaid expenses"),
            ("ASSET_CURRENT_INPUT_TAX_RECEIVABLE", "Input tax receivable"),
            ("ASSET_CURRENT_DEPOSITS_AND_ADVANCES", "Deposits and advances"),
            ("ASSET_CURRENT_SHORT_TERM_INVESTMENTS", "Short-term investments"),
            ("ASSET_CURRENT_CHEQUES_ON_HAND", "Cheques on hand"),
            ("ASSET_CURRENT_OTHER_CURRENT_ASSETS", "Other current assets"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "ASSET",
        "ASSET_NON_CURRENT",
        &[
            ("ASSET_NON_CURRENT_LAND", "Land"),
            ("ASSET_NON_CURRENT_BUILDINGS", "Buildings"),
            ("ASSET_NON_CURRENT_LEASEHOLD_IMPROVEMENTS", "Leasehold improvements"),
            ("ASSET_NON_CURRENT_FURNITURE_AND_FIXTURES", "Furniture and fixtures"),
            ("ASSET_NON_CURRENT_OFFICE_EQUIPMENT", "Office equipment"),
            ("ASSET_NON_CURRENT_COMPUTER_HARDWARE", "Computer hardware"),
            ("ASSET_NON_CURRENT_COMPUTER_SOFTWARE", "Computer software"),
            ("ASSET_NON_CURRENT_VEHICLES", "Vehicles"),
            ("ASSET_NON_CURRENT_MACHINERY", "Machinery"),
            ("ASSET_NON_CURRENT_RIGHT_OF_USE_ASSET", "Right-of-use asset"),
            ("ASSET_NON_CURRENT_LONG_TERM_DEPOSITS", "Long-term deposits"),
            ("ASSET_NON_CURRENT_LONG_TERM_INVESTMENTS", "Long-term investments"),
            ("ASSET_NON_CURRENT_SECURITY_DEPOSITS", "Security deposits"),
            ("ASSET_NON_CURRENT_ACCUMULATED_DEPRECIATION_NON_CURRENT_ASSETS", "Accumulated depreciation - non-current assets"),
            ("ASSET_NON_CURRENT_CAPITAL_WORK_IN_PROGRESS", "Capital work in progress"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "ASSET",
        "ASSET_OTHER",
        &[
            ("ASSET_OTHER_DEFERRED_TAX_ASSET", "Deferred tax asset"),
            ("ASSET_OTHER_GOODWILL", "Goodwill"),
            ("ASSET_OTHER_INTANGIBLE_ASSETS", "Intangible assets"),
            ("ASSET_OTHER_PATENT", "Patent"),
            ("ASSET_OTHER_TRADEMARK", "Trademark"),
            ("ASSET_OTHER_FRANCHISE", "Franchise"),
            ("ASSET_OTHER_SOFTWARE_DEVELOPMENT_COSTS", "Software development costs"),
            ("ASSET_OTHER_PRE_OPERATING_EXPENSES", "Pre-operating expenses"),
            ("ASSET_OTHER_REIMBURSEMENT_RECEIVABLE", "Reimbursement receivable"),
            ("ASSET_OTHER_EMPLOYEE_ADVANCES", "Employee advances"),
            ("ASSET_OTHER_LOANS_TO_STAFF", "Loans to staff"),
            ("ASSET_OTHER_INTERCOMPANY_RECEIVABLES", "Intercompany receivables"),
            ("ASSET_OTHER_OTHER_NON_CURRENT_ASSETS", "Other non-current assets"),
            ("ASSET_OTHER_SUSPENSE_ASSET", "Suspense asset"),
            ("ASSET_OTHER_MISCELLANEOUS_ASSET", "Miscellaneous asset"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "ASSET",
        "ASSET_CONTRA",
        &[
            ("ASSET_CONTRA_ACCUMULATED_DEPRECIATION_BUILDINGS", "Accumulated depreciation - buildings"),
            ("ASSET_CONTRA_ACCUMULATED_DEPRECIATION_VEHICLES", "Accumulated depreciation - vehicles"),
            ("ASSET_CONTRA_ACCUMULATED_DEPRECIATION_EQUIPMENT", "Accumulated depreciation - equipment"),
            ("ASSET_CONTRA_ALLOWANCE_FOR_DOUBTFUL_ACCOUNTS", "Allowance for doubtful accounts"),
            ("ASSET_CONTRA_INVENTORY_OBSOLESCENCE_RESERVE", "Inventory obsolescence reserve"),
            ("ASSET_CONTRA_PREPAYMENT_ADJUSTMENT", "Prepayment adjustment"),
            ("ASSET_CONTRA_ASSET_IMPAIRMENT_RESERVE", "Asset impairment reserve"),
            ("ASSET_CONTRA_TAX_PROVISION_ASSET", "Tax provision asset"),
            ("ASSET_CONTRA_CASH_OVER_AND_SHORT", "Cash over and short"),
            ("ASSET_CONTRA_WRITE_OFF_RESERVE", "Write-off reserve"),
            ("ASSET_CONTRA_CONTRA_RECEIVABLE", "Contra receivable"),
            ("ASSET_CONTRA_CONTRA_INVENTORY", "Contra inventory"),
            ("ASSET_CONTRA_CONTRA_PREPAID_EXPENSES", "Contra prepaid expenses"),
            ("ASSET_CONTRA_CONTRA_FIXED_ASSETS", "Contra fixed assets"),
            ("ASSET_CONTRA_CONTRA_OTHER_ASSETS", "Contra other assets"),
        ],
    );

    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "LIABILITY",
        "LIABILITY_CURRENT",
        &[
            ("LIABILITY_CURRENT_ACCOUNTS_PAYABLE", "Accounts payable"),
            ("LIABILITY_CURRENT_TRADE_PAYABLES", "Trade payables"),
            ("LIABILITY_CURRENT_BILLS_PAYABLE", "Bills payable"),
            ("LIABILITY_CURRENT_ACCRUED_EXPENSES", "Accrued expenses"),
            ("LIABILITY_CURRENT_SALARIES_PAYABLE", "Salaries payable"),
            ("LIABILITY_CURRENT_WAGES_PAYABLE", "Wages payable"),
            ("LIABILITY_CURRENT_GST_OUTPUT_TAX", "GST output tax"),
            ("LIABILITY_CURRENT_SALES_TAX_PAYABLE", "Sales tax payable"),
            ("LIABILITY_CURRENT_WITHHOLDING_TAX_PAYABLE", "Withholding tax payable"),
            ("LIABILITY_CURRENT_CUSTOMER_ADVANCES", "Customer advances"),
            ("LIABILITY_CURRENT_UNEARNED_REVENUE", "Unearned revenue"),
            ("LIABILITY_CURRENT_SHORT_TERM_LOAN", "Short-term loan"),
            ("LIABILITY_CURRENT_BANK_OVERDRAFT", "Bank overdraft"),
            ("LIABILITY_CURRENT_CREDIT_CARD_PAYABLE", "Credit card payable"),
            ("LIABILITY_CURRENT_CURRENT_PORTION_OF_LONG_TERM_DEBT", "Current portion of long-term debt"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "LIABILITY",
        "LIABILITY_NON_CURRENT",
        &[
            ("LIABILITY_NON_CURRENT_LONG_TERM_LOAN", "Long-term loan"),
            ("LIABILITY_NON_CURRENT_TERM_LOAN", "Term loan"),
            ("LIABILITY_NON_CURRENT_LEASE_LIABILITY", "Lease liability non-current"),
            ("LIABILITY_NON_CURRENT_DEFERRED_TAX_LIABILITY", "Deferred tax liability"),
            ("LIABILITY_NON_CURRENT_BONDS_PAYABLE", "Bonds payable"),
            ("LIABILITY_NON_CURRENT_NOTES_PAYABLE", "Notes payable"),
            ("LIABILITY_NON_CURRENT_DEBENTURES", "Debentures"),
            ("LIABILITY_NON_CURRENT_PENSION_OBLIGATION", "Pension obligation"),
            ("LIABILITY_NON_CURRENT_SECURITY_DEPOSITS_RECEIVED", "Security deposits received"),
            ("LIABILITY_NON_CURRENT_DEFERRED_REVENUE", "Deferred revenue non-current"),
            ("LIABILITY_NON_CURRENT_INTERCOMPANY_PAYABLE_NON_CURRENT", "Intercompany payable non-current"),
            ("LIABILITY_NON_CURRENT_RETIREMENT_BENEFIT_OBLIGATION", "Retirement benefit obligation"),
            ("LIABILITY_NON_CURRENT_LONG_TERM_PROVISIONS", "Long-term provisions"),
            ("LIABILITY_NON_CURRENT_OTHER_LONG_TERM_LIABILITIES", "Other long-term liabilities"),
            ("LIABILITY_NON_CURRENT_CURRENT_PORTION_RECLASS", "Current portion reclass"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "LIABILITY",
        "LIABILITY_OTHER",
        &[
            ("LIABILITY_OTHER_PROVISIONS", "Provisions"),
            ("LIABILITY_OTHER_WARRANTY_PROVISION", "Warranty provision"),
            ("LIABILITY_OTHER_LEGAL_PROVISION", "Legal provision"),
            ("LIABILITY_OTHER_TAX_PROVISION", "Tax provision"),
            ("LIABILITY_OTHER_DIVIDEND_PAYABLE", "Dividend payable"),
            ("LIABILITY_OTHER_INTEREST_PAYABLE", "Interest payable"),
            ("LIABILITY_OTHER_ROYALTIES_PAYABLE", "Royalties payable"),
            ("LIABILITY_OTHER_FRANCHISE_FEE_PAYABLE", "Franchise fee payable"),
            ("LIABILITY_OTHER_CUSTOMER_DEPOSITS", "Customer deposits"),
            ("LIABILITY_OTHER_EMPLOYEE_BENEFIT_PAYABLE", "Employee benefit payable"),
            ("LIABILITY_OTHER_IMPORT_DUTY_PAYABLE", "Import duty payable"),
            ("LIABILITY_OTHER_CUSTOMS_DUTY_PAYABLE", "Customs duty payable"),
            ("LIABILITY_OTHER_EXCISE_DUTY_PAYABLE", "Excise duty payable"),
            ("LIABILITY_OTHER_OTHER_STATUTORY_LIABILITIES", "Other statutory liabilities"),
            ("LIABILITY_OTHER_MISCELLANEOUS_PAYABLES", "Miscellaneous payables"),
        ],
    );

    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EQUITY",
        "EQUITY_CAPITAL",
        &[
            ("EQUITY_CAPITAL_SHARE_CAPITAL", "Share capital"),
            ("EQUITY_CAPITAL_COMMON_STOCK", "Common stock"),
            ("EQUITY_CAPITAL_PREFERRED_STOCK", "Preferred stock"),
            ("EQUITY_CAPITAL_PAID_IN_CAPITAL", "Paid-in capital"),
            ("EQUITY_CAPITAL_OWNERS_CAPITAL", "Owner's capital"),
            ("EQUITY_CAPITAL_PARTNER_CAPITAL", "Partner capital"),
            ("EQUITY_CAPITAL_MEMBERSHIP_CAPITAL", "Membership capital"),
            ("EQUITY_CAPITAL_OPENING_BALANCE_EQUITY", "Opening balance equity"),
            ("EQUITY_CAPITAL_TREASURY_SHARES", "Treasury shares"),
            ("EQUITY_CAPITAL_SHARE_PREMIUM", "Share premium"),
            ("EQUITY_CAPITAL_CAPITAL_CONTRIBUTION", "Capital contribution"),
            ("EQUITY_CAPITAL_CAPITAL_RESERVE", "Capital reserve"),
            ("EQUITY_CAPITAL_FOREIGN_CURRENCY_TRANSLATION_RESERVE", "Foreign currency translation reserve"),
            ("EQUITY_CAPITAL_REVALUATION_RESERVE", "Revaluation reserve"),
            ("EQUITY_CAPITAL_EQUITY_ADJUSTMENT_RESERVE", "Equity adjustment reserve"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EQUITY",
        "EQUITY_RESERVE",
        &[
            ("EQUITY_RESERVE_GENERAL_RESERVE", "General reserve"),
            ("EQUITY_RESERVE_STATUTORY_RESERVE", "Statutory reserve"),
            ("EQUITY_RESERVE_LEGAL_RESERVE", "Legal reserve"),
            ("EQUITY_RESERVE_GENERAL_SURPLUS", "General surplus"),
            ("EQUITY_RESERVE_SPECIFIC_RESERVE", "Specific reserve"),
            ("EQUITY_RESERVE_DIVIDEND_EQUALIZATION_RESERVE", "Dividend equalization reserve"),
            ("EQUITY_RESERVE_MERGER_RESERVE", "Merger reserve"),
            ("EQUITY_RESERVE_DEVELOPMENT_RESERVE", "Development reserve"),
            ("EQUITY_RESERVE_CONTINGENCY_RESERVE", "Contingency reserve"),
            ("EQUITY_RESERVE_INVESTMENT_FLUCTUATION_RESERVE", "Investment fluctuation reserve"),
            ("EQUITY_RESERVE_REPLACEMENT_RESERVE", "Replacement reserve"),
            ("EQUITY_RESERVE_REDEMPTION_RESERVE", "Redemption reserve"),
            ("EQUITY_RESERVE_CAPITAL_REDEMPTION_RESERVE", "Capital redemption reserve"),
            ("EQUITY_RESERVE_TRANSLATION_RESERVE", "Translation reserve"),
            ("EQUITY_RESERVE_OTHER_RESERVE", "Other reserve"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EQUITY",
        "EQUITY_RETAINED",
        &[
            ("EQUITY_RETAINED_RETAINED_EARNINGS", "Retained earnings"),
            ("EQUITY_RETAINED_CURRENT_YEAR_PROFIT", "Current year profit"),
            ("EQUITY_RETAINED_CURRENT_YEAR_LOSS", "Current year loss"),
            ("EQUITY_RETAINED_PRIOR_PERIOD_ADJUSTMENT", "Prior period adjustment"),
            ("EQUITY_RETAINED_APPROPRIATIONS", "Appropriations"),
            ("EQUITY_RETAINED_DIVIDENDS_DECLARED", "Dividends declared"),
            ("EQUITY_RETAINED_DIVIDENDS_PAID", "Dividends paid"),
            ("EQUITY_RETAINED_OWNER_DRAWINGS", "Owner drawings"),
            ("EQUITY_RETAINED_PARTNER_DRAWINGS", "Partner drawings"),
            ("EQUITY_RETAINED_SHAREHOLDER_DISTRIBUTION", "Shareholder distribution"),
            ("EQUITY_RETAINED_ACCUMULATED_DEFICIT", "Accumulated deficit"),
            ("EQUITY_RETAINED_OPENING_RETAINED_EARNINGS", "Opening retained earnings"),
            ("EQUITY_RETAINED_CLOSING_RETAINED_EARNINGS", "Closing retained earnings"),
            ("EQUITY_RETAINED_PROFIT_AND_LOSS_APPROPRIATION", "Profit and loss appropriation"),
            ("EQUITY_RETAINED_RESERVE_TRANSFERS", "Reserve transfers"),
        ],
    );

    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "REVENUE",
        "REVENUE_PRODUCT",
        &[
            ("REVENUE_PRODUCT_PRODUCT_SALES", "Product sales"),
            ("REVENUE_PRODUCT_DOMESTIC_SALES", "Domestic sales"),
            ("REVENUE_PRODUCT_EXPORT_SALES", "Export sales"),
            ("REVENUE_PRODUCT_ONLINE_SALES", "Online sales"),
            ("REVENUE_PRODUCT_RETAIL_SALES", "Retail sales"),
            ("REVENUE_PRODUCT_WHOLESALE_SALES", "Wholesale sales"),
            ("REVENUE_PRODUCT_PROJECT_SALES", "Project sales"),
            ("REVENUE_PRODUCT_INSTALLATION_REVENUE", "Installation revenue"),
            ("REVENUE_PRODUCT_MAINTENANCE_REVENUE", "Maintenance revenue"),
            ("REVENUE_PRODUCT_WARRANTY_REVENUE", "Warranty revenue"),
            ("REVENUE_PRODUCT_DELIVERY_REVENUE", "Delivery revenue"),
            ("REVENUE_PRODUCT_OTHER_SALES_REVENUE", "Other sales revenue"),
            ("REVENUE_PRODUCT_DISCOUNT_ALLOWED", "Discount allowed"),
            ("REVENUE_PRODUCT_SALES_RETURNS", "Sales returns"),
            ("REVENUE_PRODUCT_REBATE_REVENUE", "Rebate revenue"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "REVENUE",
        "REVENUE_SERVICE",
        &[
            ("REVENUE_SERVICE_CONSULTING_INCOME", "Consulting income"),
            ("REVENUE_SERVICE_PROFESSIONAL_SERVICES", "Professional services"),
            ("REVENUE_SERVICE_SUPPORT_SERVICES", "Support services"),
            ("REVENUE_SERVICE_AMC_INCOME", "AMC income"),
            ("REVENUE_SERVICE_TRAINING_INCOME", "Training income"),
            ("REVENUE_SERVICE_AUDIT_INCOME", "Audit income"),
            ("REVENUE_SERVICE_DESIGN_INCOME", "Design income"),
            ("REVENUE_SERVICE_DEVELOPMENT_INCOME", "Development income"),
            ("REVENUE_SERVICE_MAINTENANCE_INCOME", "Maintenance income"),
            ("REVENUE_SERVICE_INSTALLATION_INCOME", "Installation income"),
            ("REVENUE_SERVICE_CUSTOMIZATION_INCOME", "Customization income"),
            ("REVENUE_SERVICE_MANAGED_SERVICES_INCOME", "Managed services income"),
            ("REVENUE_SERVICE_OUTSOURCING_INCOME", "Outsourcing income"),
            ("REVENUE_SERVICE_SUBSCRIPTION_FEES", "Subscription fees"),
            ("REVENUE_SERVICE_ONBOARDING_FEES", "Onboarding fees"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "REVENUE",
        "REVENUE_OTHER",
        &[
            ("REVENUE_OTHER_LICENSE_INCOME", "License income"),
            ("REVENUE_OTHER_COMMISSION_INCOME", "Commission income"),
            ("REVENUE_OTHER_SERVICE_CHARGES", "Service charges"),
            ("REVENUE_OTHER_FREIGHT_RECOVERY", "Freight recovery"),
            ("REVENUE_OTHER_LATE_FEE_INCOME", "Late fee income"),
            ("REVENUE_OTHER_RENTAL_INCOME", "Rental income"),
            ("REVENUE_OTHER_SCRAP_SALES", "Scrap sales"),
            ("REVENUE_OTHER_DISCOUNT_RECEIVED", "Discount received"),
            ("REVENUE_OTHER_GRANT_INCOME", "Grant income"),
            ("REVENUE_OTHER_SUBSIDY_INCOME", "Subsidy income"),
            ("REVENUE_OTHER_REBATE_INCOME", "Rebate income"),
            ("REVENUE_OTHER_MILEAGE_INCOME", "Mileage income"),
            ("REVENUE_OTHER_MISCELLANEOUS_OPERATING_INCOME", "Miscellaneous operating income"),
            ("REVENUE_OTHER_USAGE_INCOME", "Usage income"),
            ("REVENUE_OTHER_UPGRADE_REVENUE", "Upgrade revenue"),
        ],
    );

    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "COGS",
        "COGS_DIRECT",
        &[
            ("COGS_DIRECT_OPENING_STOCK", "Opening stock"),
            ("COGS_DIRECT_PURCHASES", "Purchases"),
            ("COGS_DIRECT_PURCHASE_RETURNS", "Purchase returns"),
            ("COGS_DIRECT_FREIGHT_INWARD", "Freight inward"),
            ("COGS_DIRECT_IMPORT_DUTY", "Import duty"),
            ("COGS_DIRECT_CUSTOMS_CLEARING", "Customs clearing"),
            ("COGS_DIRECT_DIRECT_MATERIAL_CONSUMPTION", "Direct material consumption"),
            ("COGS_DIRECT_PACKAGING_MATERIALS", "Packaging materials"),
            ("COGS_DIRECT_RAW_MATERIAL_CONSUMED", "Raw material consumed"),
            ("COGS_DIRECT_SUBCONTRACT_PURCHASE", "Subcontract purchase"),
            ("COGS_DIRECT_DIRECT_CONSUMABLES", "Direct consumables"),
            ("COGS_DIRECT_MANUFACTURING_SUPPLIES", "Manufacturing supplies"),
            ("COGS_DIRECT_PURCHASE_VARIANCE", "Purchase variance"),
            ("COGS_DIRECT_PURCHASE_DISCOUNT", "Purchase discount"),
            ("COGS_DIRECT_CLOSING_STOCK_ADJUSTMENT", "Closing stock adjustment"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "COGS",
        "COGS_OVERHEAD",
        &[
            ("COGS_OVERHEAD_WAGES_PRODUCTION", "Wages - production"),
            ("COGS_OVERHEAD_FACTORY_RENT", "Factory rent"),
            ("COGS_OVERHEAD_FACTORY_UTILITIES", "Factory utilities"),
            ("COGS_OVERHEAD_FACTORY_REPAIRS", "Factory repairs"),
            ("COGS_OVERHEAD_FACTORY_DEPRECIATION", "Factory depreciation"),
            ("COGS_OVERHEAD_MACHINE_MAINTENANCE", "Machine maintenance"),
            ("COGS_OVERHEAD_FACTORY_INSURANCE", "Factory insurance"),
            ("COGS_OVERHEAD_QUALITY_CONTROL", "Quality control"),
            ("COGS_OVERHEAD_PRODUCTION_SUPPLIES", "Production supplies"),
            ("COGS_OVERHEAD_POWER_AND_FUEL", "Power and fuel"),
            ("COGS_OVERHEAD_INDIRECT_LABOR", "Indirect labor"),
            ("COGS_OVERHEAD_SUPERVISOR_SALARIES", "Supervisor salaries"),
            ("COGS_OVERHEAD_FACTORY_CLEANING", "Factory cleaning"),
            ("COGS_OVERHEAD_PRODUCTION_CONSULTANCY", "Production consultancy"),
            ("COGS_OVERHEAD_MANUFACTURING_OVERHEAD_ADJUSTMENT", "Manufacturing overhead adjustment"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "COGS",
        "COGS_LOGISTICS",
        &[
            ("COGS_LOGISTICS_FREIGHT_INWARD", "Freight inward"),
            ("COGS_LOGISTICS_FREIGHT_OUTWARD", "Freight outward"),
            ("COGS_LOGISTICS_DUTIES_AND_TAXES", "Duties and taxes"),
            ("COGS_LOGISTICS_CUSTOMS_DUTY", "Customs duty"),
            ("COGS_LOGISTICS_CUSTOMS_BROKERAGE", "Customs brokerage"),
            ("COGS_LOGISTICS_PORT_CHARGES", "Port charges"),
            ("COGS_LOGISTICS_HANDLING_CHARGES", "Handling charges"),
            ("COGS_LOGISTICS_LOADING_UNLOADING", "Loading and unloading"),
            ("COGS_LOGISTICS_TRANSIT_INSURANCE", "Transit insurance"),
            ("COGS_LOGISTICS_DELIVERY_CHARGES", "Delivery charges"),
            ("COGS_LOGISTICS_PACKING_AND_CRATING", "Packing and crating"),
            ("COGS_LOGISTICS_STORAGE_CHARGES", "Storage charges"),
            ("COGS_LOGISTICS_CARTAGE_OUTWARD", "Cartage outward"),
            ("COGS_LOGISTICS_SHIPPING_CHARGES", "Shipping charges"),
            ("COGS_LOGISTICS_LOGISTICS_VARIANCE", "Logistics variance"),
        ],
    );

    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EXPENSE",
        "EXPENSE_ADMIN",
        &[
            ("EXPENSE_ADMIN_OFFICE_RENT", "Office rent"),
            ("EXPENSE_ADMIN_OFFICE_SALARIES", "Office salaries"),
            ("EXPENSE_ADMIN_OFFICE_SUPPLIES", "Office supplies"),
            ("EXPENSE_ADMIN_POSTAGE_AND_COURIER", "Postage and courier"),
            ("EXPENSE_ADMIN_TELEPHONE", "Telephone"),
            ("EXPENSE_ADMIN_INTERNET", "Internet"),
            ("EXPENSE_ADMIN_PRINTING_AND_STATIONERY", "Printing and stationery"),
            ("EXPENSE_ADMIN_OFFICE_CLEANING", "Office cleaning"),
            ("EXPENSE_ADMIN_MEMBERSHIP_FEES", "Membership fees"),
            ("EXPENSE_ADMIN_BANK_CHARGES", "Bank charges"),
            ("EXPENSE_ADMIN_AUDIT_FEES", "Audit fees"),
            ("EXPENSE_ADMIN_LEGAL_FEES", "Legal fees"),
            ("EXPENSE_ADMIN_CONSULTANCY_FEES", "Consultancy fees"),
            ("EXPENSE_ADMIN_TRAVEL_EXPENSES", "Travel expenses"),
            ("EXPENSE_ADMIN_MEETINGS_AND_ENTERTAINMENT", "Meetings and entertainment"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EXPENSE",
        "EXPENSE_SALES",
        &[
            ("EXPENSE_SALES_ADVERTISING", "Advertising"),
            ("EXPENSE_SALES_MARKETING_CAMPAIGNS", "Marketing campaigns"),
            ("EXPENSE_SALES_SALES_SALARIES", "Sales salaries"),
            ("EXPENSE_SALES_SALES_COMMISSION", "Sales commission"),
            ("EXPENSE_SALES_CUSTOMER_DISCOUNTS", "Customer discounts"),
            ("EXPENSE_SALES_MARKET_RESEARCH", "Market research"),
            ("EXPENSE_SALES_PROMOTION_EXPENSE", "Promotion expense"),
            ("EXPENSE_SALES_TRADE_SHOWS", "Trade shows"),
            ("EXPENSE_SALES_DELIVERY_EXPENSE", "Delivery expense"),
            ("EXPENSE_SALES_PACKAGING_EXPENSE", "Packaging expense"),
            ("EXPENSE_SALES_SAMPLE_EXPENSE", "Sample expense"),
            ("EXPENSE_SALES_DEALER_COMMISSION", "Dealer commission"),
            ("EXPENSE_SALES_SPONSORSHIP", "Sponsorship"),
            ("EXPENSE_SALES_WEBSITE_MARKETING", "Website marketing"),
            ("EXPENSE_SALES_CRM_EXPENSE", "CRM expense"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EXPENSE",
        "EXPENSE_PAYROLL",
        &[
            ("EXPENSE_PAYROLL_BASIC_SALARY", "Basic salary"),
            ("EXPENSE_PAYROLL_HOUSE_RENT_ALLOWANCE", "House rent allowance"),
            ("EXPENSE_PAYROLL_MEDICAL_ALLOWANCE", "Medical allowance"),
            ("EXPENSE_PAYROLL_BONUS_EXPENSE", "Bonus expense"),
            ("EXPENSE_PAYROLL_OVERTIME", "Overtime"),
            ("EXPENSE_PAYROLL_EMPLOYER_PROVIDENT_FUND", "Employer provident fund"),
            ("EXPENSE_PAYROLL_EMPLOYER_SOCIAL_SECURITY", "Employer social security"),
            ("EXPENSE_PAYROLL_GRATUITY_EXPENSE", "Gratuity expense"),
            ("EXPENSE_PAYROLL_LEAVE_ENCASHMENT", "Leave encashment"),
            ("EXPENSE_PAYROLL_STAFF_WELFARE", "Staff welfare"),
            ("EXPENSE_PAYROLL_RECRUITMENT_EXPENSE", "Recruitment expense"),
            ("EXPENSE_PAYROLL_TRAINING_EXPENSE", "Training expense"),
            ("EXPENSE_PAYROLL_EMPLOYEE_INSURANCE", "Employee insurance"),
            ("EXPENSE_PAYROLL_PAYROLL_PROCESSING_FEES", "Payroll processing fees"),
            ("EXPENSE_PAYROLL_SEVERANCE_PAY", "Severance pay"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EXPENSE",
        "EXPENSE_FINANCE",
        &[
            ("EXPENSE_FINANCE_INTEREST_EXPENSE", "Interest expense"),
            ("EXPENSE_FINANCE_LOAN_PROCESSING_FEE", "Loan processing fee"),
            ("EXPENSE_FINANCE_BANK_CHARGES", "Bank charges"),
            ("EXPENSE_FINANCE_LATE_PAYMENT_CHARGES", "Late payment charges"),
            ("EXPENSE_FINANCE_FOREIGN_EXCHANGE_LOSS", "Foreign exchange loss"),
            ("EXPENSE_FINANCE_DISCOUNTING_CHARGES", "Discounting charges"),
            ("EXPENSE_FINANCE_CASH_HANDLING_CHARGES", "Cash handling charges"),
            ("EXPENSE_FINANCE_CREDIT_CARD_CHARGES", "Credit card charges"),
            ("EXPENSE_FINANCE_FINANCING_FEES", "Financing fees"),
            ("EXPENSE_FINANCE_LEASE_INTEREST", "Lease interest"),
            ("EXPENSE_FINANCE_GUARANTEE_FEES", "Guarantee fees"),
            ("EXPENSE_FINANCE_HEDGING_LOSS", "Hedging loss"),
            ("EXPENSE_FINANCE_FACTORING_CHARGES", "Factoring charges"),
            ("EXPENSE_FINANCE_CHEQUE_BOUNCING_CHARGES", "Cheque bouncing charges"),
            ("EXPENSE_FINANCE_OTHER_FINANCE_COSTS", "Other finance costs"),
        ],
    );
    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "EXPENSE",
        "EXPENSE_OTHER",
        &[
            ("EXPENSE_OTHER_LOSS_ON_SALE_OF_ASSET", "Loss on sale of asset"),
            ("EXPENSE_OTHER_BAD_DEBT_EXPENSE", "Bad debt expense"),
            ("EXPENSE_OTHER_PENALTY_EXPENSE", "Penalty expense"),
            ("EXPENSE_OTHER_DONATION_EXPENSE", "Donation expense"),
            ("EXPENSE_OTHER_CHARITY_EXPENSE", "Charity expense"),
            ("EXPENSE_OTHER_PRIOR_PERIOD_EXPENSE", "Prior period expense"),
            ("EXPENSE_OTHER_WRITE_OFF_EXPENSE", "Write-off expense"),
            ("EXPENSE_OTHER_ASSET_IMPAIRMENT_LOSS", "Asset impairment loss"),
            ("EXPENSE_OTHER_INVENTORY_WRITE_DOWN", "Inventory write-down"),
            ("EXPENSE_OTHER_TAX_PENALTY", "Tax penalty"),
            ("EXPENSE_OTHER_SETTLEMENT_EXPENSE", "Settlement expense"),
            ("EXPENSE_OTHER_LITIGATION_EXPENSE", "Litigation expense"),
            ("EXPENSE_OTHER_EXTRAORDINARY_LOSS", "Extraordinary loss"),
            ("EXPENSE_OTHER_OTHER_NON_OPERATING_EXPENSE", "Other non-operating expense"),
            ("EXPENSE_OTHER_MISCELLANEOUS_EXPENSE", "Miscellaneous expense"),
        ],
    );

    add_leaf_accounts(
        &mut seeds,
        &mut next_leaf_code,
        "OTHER_INCOME",
        "OTHER_INCOME_NON_OPERATING",
        &[
            ("OTHER_INCOME_NON_OPERATING_INTEREST_INCOME", "Interest income"),
            ("OTHER_INCOME_NON_OPERATING_DIVIDEND_INCOME", "Dividend income"),
            ("OTHER_INCOME_NON_OPERATING_RENT_INCOME", "Rent income"),
            ("OTHER_INCOME_NON_OPERATING_GAIN_ON_SALE_OF_ASSET", "Gain on sale of asset"),
            ("OTHER_INCOME_NON_OPERATING_FOREIGN_EXCHANGE_GAIN", "Foreign exchange gain"),
            ("OTHER_INCOME_NON_OPERATING_COMMISSION_RECEIVED", "Commission received"),
            ("OTHER_INCOME_NON_OPERATING_SCRAP_SALES", "Scrap sales"),
            ("OTHER_INCOME_NON_OPERATING_INSURANCE_CLAIM_INCOME", "Insurance claim income"),
            ("OTHER_INCOME_NON_OPERATING_DISCOUNT_RECEIVED", "Discount received"),
            ("OTHER_INCOME_NON_OPERATING_PENALTY_INCOME", "Penalty income"),
            ("OTHER_INCOME_NON_OPERATING_RECOVERY_INCOME", "Recovery income"),
            ("OTHER_INCOME_NON_OPERATING_GRANT_INCOME", "Grant income"),
            ("OTHER_INCOME_NON_OPERATING_SUBSIDY_INCOME", "Subsidy income"),
            ("OTHER_INCOME_NON_OPERATING_REBATE_INCOME", "Rebate income"),
            ("OTHER_INCOME_NON_OPERATING_MISCELLANEOUS_INCOME", "Miscellaneous income"),
        ],
    );

    seeds
}

fn add_group(
    seeds: &mut Vec<CoaAccountSeed>,
    code: &'static str,
    key: &'static str,
    name_primary: &'static str,
    description: Option<&'static str>,
) {
    seeds.push(CoaAccountSeed {
        key,
        code: code.to_string(),
        name_primary,
        name_secondary: None,
        description,
        kind: CoaNodeKind::Group,
        parent_key: None,
        account_group_key: None,
        account_type_key: None,
        level_no: 0,
        is_posting: false,
        is_system_account: true,
    });
}

fn add_type(
    seeds: &mut Vec<CoaAccountSeed>,
    code: &'static str,
    key: &'static str,
    name_primary: &'static str,
    description: Option<&'static str>,
    parent_key: &'static str,
) {
    seeds.push(CoaAccountSeed {
        key,
        code: code.to_string(),
        name_primary,
        name_secondary: None,
        description,
        kind: CoaNodeKind::Type,
        parent_key: Some(parent_key),
        account_group_key: None,
        account_type_key: None,
        level_no: 1,
        is_posting: false,
        is_system_account: true,
    });
}

fn add_leaf_accounts(
    seeds: &mut Vec<CoaAccountSeed>,
    next_code: &mut u32,
    account_group_key: &'static str,
    account_type_key: &'static str,
    accounts: &[(&'static str, &'static str)],
) {
    for &(key, name_primary) in accounts {
        seeds.push(CoaAccountSeed {
            key,
            code: next_code.to_string(),
            name_primary,
            name_secondary: None,
            description: None,
            kind: CoaNodeKind::Account,
            parent_key: Some(account_type_key),
            account_group_key: Some(account_group_key),
            account_type_key: Some(account_type_key),
            level_no: 2,
            is_posting: true,
            is_system_account: false,
        });
        *next_code += 1;
    }
}
