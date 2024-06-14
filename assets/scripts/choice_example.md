# Choices
## Docent
Open the choices box.[^wait]
* [Fox walking](choice_example.md#Walking)
* [Fox stopping](choice_example.md#Stopping)
* [Fox running](choice_example.md#Running)
* [Close talk](choice_example.md#Closing)

# Walking
[^feed]
The fox is walking[^wait]
[^signal(Fox_walk)]
[jump](choice_example.md#Choices)

# Stopping
[^feed]
The fox is stopping[^wait]
[^signal(Fox_stop)]
[jump](choice_example.md#Choices)

# Running
[^feed]
The fox is running[^wait]
[^signal(Fox_run)]
[jump](choice_example.md#Choices)

# Closing
[^feed]
This dialog box is closing[^wait][^close]

[^wait]: Waiting for input  
[^feed]: Force feeding
[^signal(Fox_walk)]: Play fox walking motion  
[^signal(Fox_stop)]: Play fox searching motion  
[^signal(Fox_run)]: Play fox running motion  
[^close]: Close dialog box
