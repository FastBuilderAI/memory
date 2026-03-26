# Logical_Reasoning Knowledge Base

## [ID: ATF_S_0]
**Action:** Process_Natalia_Clips
**Input:** {Context}
**Logic:** Natalia sold clips to 48 of her friends in April, and then she sold half as many clips in May.
**Data_Connections:** [natalia], [clips], [friends]
**Access:** Role_Analyst
**Events:** Trigger_On_Natalia_Modify

## [ID: ATF_S_1]
**Action:** Process_Clips_Natalia
**Input:** {Context}
**Logic:** How many clips did Natalia sell altogether in April and May?
**Data_Connections:** [clips], [natalia], [altogether]
**Access:** Role_Operator
**Events:** Trigger_On_Clips_Modify

## [ID: ATF_S_2]
**Action:** Process_Weng_Hour
**Input:** {Context}
**Logic:** Weng earns $12 an hour for babysitting.
**Data_Connections:** [earns], [babysitting]
**Access:** Role_Operator
**Events:** Trigger_On_Weng_Modify

## [ID: ATF_S_3]
**Action:** Process_Yesterday_Minutes
**Input:** {Context}
**Logic:** Yesterday, she just did 50 minutes of babysitting.
**Data_Connections:** [yesterday], [minutes], [babysitting]
**Access:** Role_Analyst
**Events:** Trigger_On_Yesterday_Modify

## [ID: ATF_S_4]
**Action:** Parse_4
**Input:** {Context}
**Logic:** How much did she earn?
**Data_Connections:** [Record_4]
**Access:** Role_Operator
**Events:** Trigger_Default

## [ID: ATF_S_5]
**Action:** Process_Betty_Money
**Input:** {Context}
**Logic:** Betty is saving money for a new wallet which costs $100.
**Data_Connections:** [betty], [saving], [money]
**Access:** Role_Operator
**Events:** Trigger_On_Betty_Modify

## [ID: ATF_S_6]
**Action:** Process_Betty_Half
**Input:** {Context}
**Logic:** Betty has only half of the money she needs.
**Data_Connections:** [betty], [money], [needs]
**Access:** Role_Analyst
**Events:** Trigger_On_Betty_Modify

## [ID: ATF_S_7]
**Action:** Process_Parents_Purpose
**Input:** {Context}
**Logic:** Her parents decided to give her $15 for that purpose, and her grandparents twice as much as her parents.
**Data_Connections:** [parents], [decided], [purpose]
**Access:** Role_Operator
**Events:** Trigger_On_Parents_Modify

## [ID: ATF_S_8]
**Action:** Process_Money_Betty
**Input:** {Context}
**Logic:** How much more money does Betty need to buy the wallet?
**Data_Connections:** [money], [betty], [wallet]
**Access:** Role_Operator
**Events:** Trigger_On_Money_Modify

## [ID: ATF_S_9]
**Action:** Process_Julie_Book
**Input:** {Context}
**Logic:** Julie is reading a 120-page book.
**Data_Connections:** [julie], [reading], [120page]
**Access:** Role_Analyst
**Events:** Trigger_On_Julie_Modify

## [ID: ATF_S_10]
**Action:** Process_Yesterday_Pages
**Input:** {Context}
**Logic:** Yesterday, she was able to read 12 pages and today, she read twice as many pages as yesterday.
**Data_Connections:** [yesterday], [pages], [today]
**Access:** Role_Operator
**Events:** Trigger_On_Yesterday_Modify

## [ID: ATF_S_11]
**Action:** Process_Half_Pages
**Input:** {Context}
**Logic:** If she wants to read half of the remaining pages tomorrow, how many pages should she read?
**Data_Connections:** [wants], [remaining], [pages]
**Access:** Role_Operator
**Events:** Trigger_On_Half_Modify

## [ID: ATF_S_12]
**Action:** Process_James_Letter
**Input:** {Context}
**Logic:** James writes a 3-page letter to 2 different friends twice a week.
**Data_Connections:** [james], [writes], [3page]
**Access:** Role_Analyst
**Events:** Trigger_On_James_Modify

## [ID: ATF_S_13]
**Action:** Process_Pages_Year
**Input:** {Context}
**Logic:** How many pages does he write a year?
**Data_Connections:** [pages], [write]
**Access:** Role_Operator
**Events:** Trigger_On_Pages_Modify

## [ID: ATF_S_14]
**Action:** Process_Mark_Garden
**Input:** {Context}
**Logic:** Mark has a garden with flowers.
**Data_Connections:** [garden], [flowers]
**Access:** Role_Operator
**Events:** Trigger_On_Mark_Modify

## [ID: ATF_S_15]
**Action:** Process_Plants_Colors
**Input:** {Context}
**Logic:** He planted plants of three different colors in it.
**Data_Connections:** [planted], [plants], [three]
**Access:** Role_Analyst
**Events:** Trigger_On_Plants_Modify

## [ID: ATF_S_16]
**Action:** Process_Purple
**Input:** {Context}
**Logic:** Ten of them are yellow, and there are 80% more of those in purple.
**Data_Connections:** [yellow], [purple]
**Access:** Role_Operator
**Events:** Trigger_On_Purple_Modify

## [ID: ATF_S_17]
**Action:** Process_Flowers_Flowers
**Input:** {Context}
**Logic:** There are only 25% as many green flowers as there are yellow and purple flowers.
**Data_Connections:** [green], [flowers], [yellow]
**Access:** Role_Operator
**Events:** Trigger_On_Flowers_Modify

## [ID: ATF_S_18]
**Action:** Process_Flowers_Mark
**Input:** {Context}
**Logic:** How many flowers does Mark have in his garden?
**Data_Connections:** [flowers], [garden]
**Access:** Role_Analyst
**Events:** Trigger_On_Flowers_Modify

## [ID: ATF_S_19]
**Action:** Process_Albert_Day
**Input:** {Context}
**Logic:** Albert is wondering how much pizza he can eat in one day.
**Data_Connections:** [albert], [wondering], [pizza]
**Access:** Role_Operator
**Events:** Trigger_On_Albert_Modify

