import graphene
from graphene_django.debug import DjangoDebug
from graphene import ObjectType, Field, Schema
import accounts.schema
import categories.schema
import custom_user.schema
import transactions.schema


class Query(
    accounts.schema.Query,
    categories.schema.Query,
    custom_user.schema.Query,
    transactions.schema.Query,
    ObjectType,
):
    debug = Field(DjangoDebug, name="_debug")
    pass


class Mutation(
    accounts.schema.Mutation,
    categories.schema.Mutation,
    custom_user.schema.Mutation,
    # transactions.schema.Mutation,
    ObjectType,
):
    pass


schema = Schema(query=Query, mutation=Mutation)
