# Knowledge Base

## [ID: ATF_REF_01]
**Action:** Provide_Reimbursement_Logic
**Input:** {Expense_Type, Receipt_Image}
**Logic:** If Expense_Type == 'Travel', apply Rule_X.
**Context_Links:** [ATF_REF_02]

## [ID: ATF_REF_02]
**Action:** Travel Policy
**Input:** {}
**Logic:** All travel must be approved.
**Context_Links:** [ATF_REF_01]
