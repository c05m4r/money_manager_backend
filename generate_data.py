import os
import django
from decimal import Decimal
import random
from faker import Faker

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "core.settings")
django.setup()

from custom_user.models import CustomUser
from accounts.models import Account
from categories.models import Category
from transactions.models import Transaction

fake = Faker()


def generate_data():
    users = []
    for _ in range(200):
        while True:
            username = fake.unique.user_name()
            if not CustomUser.objects.filter(username=username).exists():
                break
        users.append(
            CustomUser.objects.create_user(username=username, password="password")
        )

    accounts = [Account.objects.create(name=fake.company()) for _ in range(30)]
    for account in accounts:
        account.users.set(random.sample(users, random.randint(1, len(users))))

    categories = []
    while len(categories) < 15:
        name = fake.unique.word()
        category, created = Category.objects.get_or_create(name=name)
        if created:
            categories.append(category)

    for _ in range(500):
        Transaction.objects.create(
            amount=Decimal(fake.random_number(digits=4)),
            transaction_type=random.choice(["IN", "OUT"]),
            category=random.choice(categories),
            account=random.choice(accounts),
        )

    print("Datos de prueba creados exitosamente.")


if __name__ == "__main__":
    generate_data()
