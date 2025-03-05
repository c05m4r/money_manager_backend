from graphene import relay, ObjectType, Schema
from graphene_django import DjangoObjectType
from graphene_django.filter import DjangoFilterConnectionField
from transactions.models import Transaction


class TransactionNode(DjangoObjectType):
    class Meta:
        model = Transaction
        filter_fields = "__all__"
        interfaces = (relay.Node,)


class Query(ObjectType):
    transaction = relay.Node.Field(TransactionNode)
    all_transactions = DjangoFilterConnectionField(TransactionNode)


schema = Schema(query=Query, types=[TransactionNode])
