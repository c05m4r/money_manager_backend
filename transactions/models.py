from django.db import models
from categories.models import Category
from accounts.models import Account
from core.models import BaseModel, SoftDeleteModel


class Transaction(BaseModel, SoftDeleteModel):
    TRANSACTION_TYPE_CHOICES = [
        ("IN", "Income"),
        ("OUT", "Expense"),
    ]

    amount = models.DecimalField(max_digits=10, decimal_places=2, verbose_name="Amount")
    transaction_type = models.CharField(
        max_length=3, choices=TRANSACTION_TYPE_CHOICES, verbose_name="Transaction Type"
    )
    category = models.ForeignKey(
        Category, on_delete=models.DO_NOTHING, verbose_name="Category"
    )
    account = models.ForeignKey(
        Account, on_delete=models.PROTECT, verbose_name="Account"
    )

    def __str__(self):
        return self.transaction_type
