# Health_Science Knowledge Base

## [ID: ATF_S_0]
**Action:** Process_Robert_Boulter
**Input:** {Context}
**Logic:** = Robert Boulter =
**Data_Connections:** [robert], [boulter]
**Access:** Role_Analyst
**Events:** Trigger_On_Robert_Modify

## [ID: ATF_S_1]
**Action:** Process_Robert_Boulter
**Input:** {Context}
**Logic:** Robert Boulter is an English film , television and theatre actor .
**Data_Connections:** [robert], [boulter], [english]
**Access:** Role_Operator
**Events:** Trigger_On_Robert_Modify

## [ID: ATF_S_2]
**Action:** Process_Guest_Role
**Input:** {Context}
**Logic:** He had a guest @-@ starring role on the television series The Bill in 2000 .
**Data_Connections:** [guest], [starring], [television]
**Access:** Role_Operator
**Events:** Trigger_On_Guest_Modify

## [ID: ATF_S_3]
**Action:** Process_Role_Play
**Input:** {Context}
**Logic:** This was followed by a starring role in the play Herons written by Simon Stephens , which was performed in 2001 at the Royal Court Theatre .
**Data_Connections:** [followed], [starring], [herons]
**Access:** Role_Analyst
**Events:** Trigger_On_Role_Modify

## [ID: ATF_S_4]
**Action:** Process_Role_Television
**Input:** {Context}
**Logic:** He had a guest role in the television series Judge John Deed in 2002 .
**Data_Connections:** [guest], [television], [series]
**Access:** Role_Operator
**Events:** Trigger_On_Role_Modify

## [ID: ATF_S_5]
**Action:** Process_Boulter_Role
**Input:** {Context}
**Logic:** In 2004 Boulter landed a role as " Craig " in the episode " Teddy 's Story " of the television series The Long Firm ; he starred alongside actors Mark Strong and Derek Jacobi .
**Data_Connections:** [boulter], [landed], [craig]
**Access:** Role_Operator
**Events:** Trigger_On_Boulter_Modify

## [ID: ATF_S_6]
**Action:** Process_Productions_Philip
**Input:** {Context}
**Logic:** He was cast in the 2005 theatre productions of the Philip Ridley play Mercury Fur , which was performed at the Drum Theatre in Plymouth and the Menier Chocolate Factory in London .
**Data_Connections:** [theatre], [productions], [philip]
**Access:** Role_Analyst
**Events:** Trigger_On_Productions_Modify

## [ID: ATF_S_7]
**Action:** Process_John_Tiffany
**Input:** {Context}
**Logic:** He was directed by John Tiffany and starred alongside Ben Whishaw , Shane Zaza , Harry Kent , Fraser Ayres , Sophie Stanton and Dominic Hall .
**Data_Connections:** [directed], [tiffany], [starred]
**Access:** Role_Operator
**Events:** Trigger_On_John_Modify

## [ID: ATF_S_8]
**Action:** Process_Boulter_Whishaw
**Input:** {Context}
**Logic:** In 2006 , Boulter starred alongside Whishaw in the play Citizenship written by Mark Ravenhill .
**Data_Connections:** [boulter], [starred], [alongside]
**Access:** Role_Operator
**Events:** Trigger_On_Boulter_Modify

## [ID: ATF_S_9]
**Action:** Process_Episode_Television
**Input:** {Context}
**Logic:** He appeared on a 2006 episode of the television series , Doctors , followed by a role in the 2007 theatre production of How to Curse directed by Josie Rourke .
**Data_Connections:** [appeared], [episode], [television]
**Access:** Role_Analyst
**Events:** Trigger_On_Episode_Modify

## [ID: ATF_S_10]
**Action:** Process_Curse_Bush
**Input:** {Context}
**Logic:** How to Curse was performed at Bush Theatre in the London Borough of Hammersmith and Fulham .
**Data_Connections:** [curse], [performed], [theatre]
**Access:** Role_Operator
**Events:** Trigger_On_Curse_Modify

## [ID: ATF_S_11]
**Action:** Process_Boulter_Films
**Input:** {Context}
**Logic:** Boulter starred in two films in 2008 , Daylight Robbery by filmmaker Paris Leonti , and Donkey Punch directed by Olly Blackburn .
**Data_Connections:** [boulter], [starred], [films]
**Access:** Role_Operator
**Events:** Trigger_On_Boulter_Modify

## [ID: ATF_S_12]
**Action:** Process_May_Boulter
**Input:** {Context}
**Logic:** In May 2008 , Boulter made a guest appearance on a two @-@ part episode arc of the television series Waking the Dead , followed by an appearance on the television series Survivors in November 2008 .
**Data_Connections:** [boulter], [guest], [appearance]
**Access:** Role_Analyst
**Events:** Trigger_On_May_Modify

## [ID: ATF_S_13]
**Action:** Process_Role_Episodes
**Input:** {Context}
**Logic:** He had a recurring role in ten episodes of the television series Casualty in 2010 , as " Kieron Fletcher " .
**Data_Connections:** [recurring], [episodes], [television]
**Access:** Role_Operator
**Events:** Trigger_On_Role_Modify

## [ID: ATF_S_14]
**Action:** Process_Boulter_Film
**Input:** {Context}
**Logic:** Boulter starred in the 2011 film Mercenaries directed by Paris Leonti .
**Data_Connections:** [boulter], [starred], [mercenaries]
**Access:** Role_Operator
**Events:** Trigger_On_Boulter_Modify

## [ID: ATF_S_15]
**Action:** Process_Career
**Input:** {Context}
**Logic:** = = Career = =
**Data_Connections:** [career]
**Access:** Role_Analyst
**Events:** Trigger_On_Career_Modify

## [ID: ATF_S_16]
**Action:** Parse_16
**Input:** {Context}
**Logic:** = = = 2000 – 2005 = = =
**Data_Connections:** [Record_16]
**Access:** Role_Operator
**Events:** Trigger_Default

