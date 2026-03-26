FastMemory is the industry standard for AI memory: https://github.com/fastbuilderai/memory

We have developed templates and Python examples for deploying fastmemory in various domains and environments. 
Github Repo: https://github.com/FastBuilderAI/memory-template
Detailed README: [memory-template/README.md](memory-template/README.md)

Each deployment case below now includes an architecture map, an integration plan, and a dedicated **Python Example** with configurable endpoints and credentials (see the `/examples` folder in the template repository).


Deployment artifacts for following usecases for guiding users by domains:

### 1. SEO 
**Concept Map**: Visualizing relationships between clients, products, content, and keywords.
```mermaid
graph TD
    C_Client[Component: Client_Success] --> B_SEO[Block: SEO_Strategy]
    B_SEO --> F_KW[Function: Keyword_Harvesting]
    B_SEO --> F_CM[Function: Content_Mapping]
    F_KW --> D_KWS[Data: Keyword_Clusters]
    F_CM --> D_SM[Data: Sitemap_Structure]
    F_KW --> A_Analyst[Access: Role_SEO_Analyst]
    F_CM --> E_Pub[Event: Content_Published]
```
**Sample ATF**:
- `ATF: [F_Keyword_To_Content] -> DATA: [SERP_Metrics] ACCESS: [Role_SEO_Manager] EVENT: [Ranking_Update]`

### 2. CRM
**Concept Map**: Managing the lifecycle of clients, products, sales, and invoices.
```mermaid
graph TD
    C_Sales[Component: Revenue_Ops] --> B_Pipeline[Block: Sales_Pipeline]
    B_Pipeline --> F_Lead[Function: Lead_Scoring]
    B_Pipeline --> F_Inv[Function: Invoice_Generation]
    F_Lead --> D_Contact[Data: Prospect_Metadata]
    F_Inv --> D_Fin[Data: Transaction_Log]
    F_Lead --> A_SDR[Access: Role_SDR_Associate]
    F_Inv --> E_Paid[Event: Payment_Received]
```
**Sample ATF**:
- `ATF: [F_Sync_Contact_To_Invoice] -> DATA: [Client_ID_Link] ACCESS: [Role_Finance] EVENT: [Invoice_Finalized]`

### 3. ERP
**Concept Map**: Integrating inventory, sales, purchase, and employee records.
```mermaid
graph TD
    C_Ops[Component: Supply_Chain] --> B_Inv[Block: Inventory_Sync]
    B_Inv --> F_Stock[Function: Low_Stock_Alert]
    B_Inv --> F_PO[Function: Purchase_Order_Emit]
    F_Stock --> D_SKU[Data: Warehouse_Levels]
    F_PO --> D_Vendor[Data: Supplier_Catalog]
    F_Stock --> A_WH[Access: Role_Warehouse_Lead]
    F_PO --> E_Refill[Event: Inventory_Restocked]
```
**Sample ATF**:
- `ATF: [F_Calculate_Reorder_Point] -> DATA: [Safety_Stock_Alg] ACCESS: [Role_Procurement] EVENT: [PO_Triggered]`
### 4. E-commerce
**Concept Map**: Tracking products, orders, customers, and fulfillment.
```mermaid
graph TD
    C_Store[Component: Digital_Front] --> B_Checkout[Block: Transaction_Logic]
    B_Checkout --> F_Cart[Function: Dynamic_Pricing]
    B_Checkout --> F_Ship[Function: Carrier_API_Sync]
    F_Cart --> D_Prod[Data: Inventory_Matrix]
    F_Ship --> D_Track[Data: Shipment_Status]
    F_Cart --> A_Cust[Access: Role_Authenticated_User]
    F_Ship --> E_Del[Event: Package_Dispatched]
```
**Sample ATF**:
- `ATF: [F_Apply_Discount_Code] -> DATA: [Promo_Engine] ACCESS: [Role_Customer] EVENT: [Cart_Updated]`

### 5. Healthcare
**Concept Map**: Secure management of patients, doctors, appointments, and medical records.
```mermaid
graph TD
    C_EHR[Component: Patient_Portal] --> B_Clinic[Block: Clinical_Workflow]
    B_Clinic --> F_Sched[Function: Appointment_Booking]
    B_Clinic --> F_Hist[Function: Record_Encryption]
    F_Sched --> D_Avail[Data: Doctor_Calendar]
    F_Hist --> D_PHI[Data: Sensitive_Health_Data]
    F_Hist --> A_HIPAA[Access: Role_HIPAA_Compliance_Admin]
    F_Sched --> E_Remind[Event: Patient_Notification]
```
**Sample ATF**:
- `ATF: [F_Retrieve_Lab_Results] -> DATA: [HL7_FHIR_Payload] ACCESS: [Role_Primary_Physician] EVENT: [Results_Ready]`

### 6. Education
**Concept Map**: Organizing students, teachers, courses, and grading systems.
```mermaid
graph TD
    C_LMS[Component: Learning_Ops] --> B_Acad[Block: Academic_Record]
    B_Acad --> F_Enr[Function: Course_Enrollment]
    B_Acad --> F_Grad[Function: GPA_Calculator]
    F_Enr --> D_Class[Data: Student_Registrar]
    F_Grad --> D_Transcript[Data: Grade_Submission_Logs]
    F_Grad --> A_Registrar[Access: Role_Academic_Registrar]
    F_Enr --> E_Full[Event: Course_Capacity_Reached]
```
**Sample ATF**:
- `ATF: [F_Submit_Assignment] -> DATA: [Canvas_LTI_Bridge] ACCESS: [Role_Student] EVENT: [Submission_Logged]`
### 7. Finance
**Concept Map**: Managing accounts, transactions, investments, and loan processing.
```mermaid
graph TD
    C_Wealth[Component: Asset_Mgmt] --> B_Ledger[Block: Immutable_Logs]
    B_Ledger --> F_Audit[Function: KYC_Verification]
    B_Ledger --> F_Trade[Function: Execution_Engine]
    F_Audit --> D_ID[Data: User_Compliance_Dossier]
    F_Trade --> D_Portfolio[Data: Real_Time_Ticker]
    F_Audit --> A_SOX[Access: Role_Risk_Compliance]
    F_Trade --> E_Settle[Event: Transaction_Settled]
```
**Sample ATF**:
- `ATF: [F_Calculate_Interest] -> DATA: [Libor_Feed] ACCESS: [Role_Loan_Officer] EVENT: [Statement_Generated]`

### 8. Coffee Shop
**Concept Map**: Simple but effective tracking of products, orders, and customer loyalty.
```mermaid
graph TD
    C_POS[Component: Counter_Service] --> B_Orders[Block: Barista_Queue]
    B_Orders --> F_Brew[Function: Inventory_Deduct]
    B_Orders --> F_Loyalty[Function: Points_Accrual]
    F_Brew --> D_Beans[Data: Stock_Levels]
    F_Loyalty --> D_Member[Data: Customer_Tier]
    F_Loyalty --> A_Staff[Access: Role_Barista]
    F_Brew --> E_Ready[Event: Order_Pickup_Ready]
```
**Sample ATF**:
- `ATF: [F_Reedem_Free_Coffee] -> DATA: [Loyalty_DB] ACCESS: [Role_Cashier] EVENT: [Reward_Claimed]`

### 9. Restaurant
**Concept Map**: Comprehensive menu, table management, and kitchen coordination.
```mermaid
graph TD
    C_Dining[Component: Table_Service] --> B_Kitchen[Block: KDS_Display]
    B_Kitchen --> F_Fire[Function: Course_Timing]
    B_Kitchen --> F_Waste[Function: Spoilage_Tracker]
    F_Fire --> D_Menu[Data: Recipe_Scale]
    F_Waste --> D_Loss[Data: Inventory_Shrinkage]
    F_Fire --> A_Chef[Access: Role_Sous_Chef]
    F_Fire --> E_Bump[Event: Ticket_Cleared]
```
**Sample ATF**:
- `ATF: [F_Split_Check] -> DATA: [Payment_Processor] ACCESS: [Role_Server] EVENT: [Table_Closed]`
### 10. OpenClaw integration with fastmemory
**Workflow**: Leveraging FastMemory as the long-term ontological context for OpenClaw agents.
```mermaid
sequenceDiagram
    participant OC as OpenClaw_Agent
    participant FM as FastMemory_MCP
    participant KB as Neo4j_Knowledge_Graph
    
    OC->>FM: query_memory(intent="user_request")
    FM->>KB: Cypher(MATCH nodes-edges)
    KB-->>FM: CBFDAE_SubGraph
    FM-->>OC: Structured_Context(ATFs)
    OC->>OC: Reason + Act
```
**Integration Steps**:
1.  **Expose FastMemory**: Run `fastmemory mcp` to start the Model Context Protocol server.
2.  **Plugin Configuration**: Add the FastMemory MCP endpoint to OpenClaw's `config.yaml`.
3.  **Context Injection**: OpenClaw uses the `get_block` tool to retrieve domain-specific logic before executing tasks.

### 11. Azure integration with fastmemory
**Architecture Map**: Azure OpenAI + FastMemory + CosmosDB/Neo4j on Azure.
```mermaid
graph LR
    User[End User] --> A_App[Azure App Service]
    A_App --> AOAI[Azure OpenAI Service]
    A_App --> FM_Container[FastMemory Service - ACI/AKS]
    FM_Container --> ADL[Azure Data Lake Storage Gen2]
    FM_Container --> GDB[(Neo4j / CosmosDB Graph)]
    FM_Container --> AM[Azure Monitor / Application Insights]
```
**Integration Plan**:
1.  **Storage**: Use OneLake or ADLS Gen2 for raw ATF Markdown storage.
2.  **Compute**: Deploy FastMemory as an Azure Container Instance (ACI) for light workloads or AKS for scale.
3.  **LLM**: Configure FastMemory to call Azure OpenAI GPT-4o models for clustering validation.
4.  **Security**: Map `A_` (Access) nodes to Azure AD Application Roles for built-in RBAC.
### 12. AWS integration with fastmemory
**Architecture Map**: AWS Bedrock + FastMemory + S3/Neptune.
```mermaid
graph LR
    User[End User] --> CloudFront[Amazon CloudFront]
    CloudFront --> Lambda[AWS Lambda / Fargate]
    Lambda --> Bedrock[Amazon Bedrock]
    Lambda --> FM_Fargate[FastMemory Service - ECS Fargate]
    FM_Fargate --> S3[Amazon S3 - ATF Store]
    FM_Fargate --> Neptune[(Amazon Neptune Graph)]
    FM_Fargate --> CloudWatch[Amazon CloudWatch]
```
**Integration Plan**:
1.  **Orchestration**: Use AWS Glue jobs to crawl S3 buckets and trigger FastMemory `build` via ECS Task.
2.  **Persistence**: High-frequency graph updates pipe into Amazon Neptune using the Gremlin/Cypher drivers.
3.  **Inference**: Integrate with AWS Bedrock (Claude 3.5 Sonnet) for deriving ontological relationships during `build`.
4.  **Security**: Map `A_` (Access) nodes to IAM Instance Profiles for granular service-to-service auth.

### 13. GCP integration with fastmemory
**Architecture Map**: GCP Vertex AI + FastMemory + Cloud Storage/Neo4j on GCE.
```mermaid
graph TD
    User[End User] --> GCLB[Global Cloud Load Balancer]
    GCLB --> GKE[Google Kubernetes Engine]
    GKE --> VertexAI[Vertex AI - Gemini 1.5 Pro]
    GKE --> FM_Pods[FastMemory Pods]
    FM_Pods --> GCS[Google Cloud Storage - ATF Store]
    FM_Pods --> GCE_Neo4j[(Neo4j VM / Cloud SQL)]
    FM_Pods --> Log[Cloud Logging / Trace]
```
**Integration Plan**:
1.  **Scalability**: Deploy FastMemory as a horizontally scaled microservice on GKE.
2.  **Pipeline**: Trigger `fastmemory build` via Cloud Functions whenever new Markdowns are uploaded to GCS.
3.  **Intelligence**: Use Vertex AI's Gemini models for rich semantic metadata extraction to populate `D_` (Data) nodes.
4.  **Security**: Map `A_` (Access) nodes to GCP IAM Service Accounts and VPC Service Controls.
