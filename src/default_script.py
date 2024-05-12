import time
import random
import string

print("Default script is creating...")
# time.sleep(0.5)
for i in range(3):
    r = ''.join(random.choices(string.ascii_letters + string.digits, k=8))
    print(f"random string--{r}")
print("end")
